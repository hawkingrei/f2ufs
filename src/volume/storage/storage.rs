use std::cmp::min;
use std::fmt::{self, Debug};
use std::sync::Arc;
use std::sync::RwLock;
use std::path::Path;

use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::trans::eid::Eid;
use crate::util::crypto::{Cipher, Cost, Crypto, Key};
use crate::util::lru::{CountMeter, Lru, Meter, PinChecker};
use crate::volume::address::Addr;
use crate::volume::storage::Storable;
use crate::BLKS_PER_FRAME;
use crate::error::{Error, Result};
use crate::util::IntoRef;

// frame cache meter, measured by frame byte size
#[derive(Debug, Default)]
struct FrameCacheMeter;

impl Meter<Vec<u8>> for FrameCacheMeter {
    #[inline]
    fn measure(&self, item: &Vec<u8>) -> isize {
        item.len() as isize
    }
}

/// Storage
pub struct Storage {
    depot: Box<Storable>,

    // block allocator
    allocator: AllocatorRef,

    // crypto context
    crypto: Crypto,
    key: Key,

    // decrypted frame cache, key is the begin block index
    frame_cache: Lru<usize, Vec<u8>, FrameCacheMeter, PinChecker<Vec<u8>>>,

    // entity address cache
    addr_cache: Lru<Eid, Addr, CountMeter<Addr>, PinChecker<Addr>>,
}

impl Storage {
    // frame cache size, in bytes
    const FRAME_CACHE_SIZE: usize = 4 * 1024 * 1024;

    // frame cache threshold size, in bytes
    // if the entity size is larger than this, its frames won't be
    // put in frame cache
    const FRAME_CACHE_THRESHOLD: usize = 512 * 1024;

    // address cache size
    const ADDRESS_CACHE_SIZE: usize = 64;

    pub fn new(uri: &str) -> Result<Self> {
        let depot: Box<Storable> = if uri.starts_with("file://") {
            let path = Path::new(&uri[7..]);
            let depot = FileStorage::new(path);
            Box::new(depot)
        } else {
            return Err(Error::InvalidUri);
        };

        let frame_cache = Lru::new(Self::FRAME_CACHE_SIZE);

        Ok(Storage {
            depot,
            allocator: Allocator::new().into_ref(),
            crypto: Crypto::default(),
            key: Key::new_empty(),
            frame_cache,
            addr_cache: Lru::new(Self::ADDRESS_CACHE_SIZE),
        })
    }

    #[inline]
    pub fn depot_mut(&mut self) -> &mut Storable {
        self.depot.deref_mut()
    }

    #[inline]
    pub fn crypto_ctx(&self) -> (&Crypto, &Key) {
        (&self.crypto, &self.key)
    }

    #[inline]
    pub fn exists(&self) -> Result<bool> {
        self.depot.exists()
    }

    #[inline]
    pub fn connect(&mut self) -> Result<()> {
        self.depot.connect()
    }

    pub fn init(&mut self, cost: Cost, cipher: Cipher) -> Result<()> {
        // create crypto and master key
        self.crypto = Crypto::new(cost, cipher)?;
        self.key = Crypto::gen_master_key();

        // initialise depot
        self.depot.init(self.crypto.clone(), self.key.derive(0))
    }

    pub fn open(&mut self, cost: Cost, cipher: Cipher, key: Key) -> Result<()> {
        self.crypto = Crypto::new(cost, cipher)?;
        self.key = key;

        // open depot
        self.depot.open(self.crypto.clone(), self.key.derive(0))
    }

    #[inline]
    pub fn close(&mut self) -> Result<()> {
        self.depot.close()
    }

    #[inline]
    pub fn allocator(&self) -> AllocatorRef {
        self.allocator.clone()
    }

    // read entity address from depot and save to address cache
    fn get_address(&mut self, id: &Eid) -> Result<Addr> {
        // get from address cache first
        if let Some(addr) = self.addr_cache.get_refresh(id) {
            return Ok(addr.clone());
        }

        // if not in the cache, load if from depot
        let buf = self.depot.get_address(id)?;
        let buf = self.crypto.decrypt(&buf, &self.key)?;
        let mut de = Deserializer::new(&buf[..]);
        let addr: Addr = Deserialize::deserialize(&mut de)?;

        // and then insert into address cache
        self.addr_cache.insert(id.clone(), addr.clone());

        Ok(addr)
    }

    // write entity address to depot
    fn put_address(&mut self, id: &Eid, addr: &Addr) -> Result<()> {
        // serialize address and encrypt address
        let mut buf = Vec::new();
        addr.serialize(&mut Serializer::new(&mut buf))?;
        let buf = self.crypto.encrypt(&buf, &self.key)?;

        // write to depot and remove address from cache
        self.depot.put_address(id, &buf)?;
        self.addr_cache.insert(id.clone(), addr.clone());

        Ok(())
    }

    // remove all blocks in a address
    fn remove_address_blocks(&mut self, addr: &Addr) -> Result<()> {
        let mut inaddr_idx = 0;
        for loc_span in addr.iter() {
            let blk_cnt = loc_span.span.cnt;

            // delete blocks
            self.depot.del_blocks(loc_span.span)?;

            let mut blk_idx = loc_span.span.begin;
            let end_idx = inaddr_idx + blk_cnt;

            while inaddr_idx < end_idx {
                let offset = inaddr_idx % BLKS_PER_FRAME;
                if offset == 0 {
                    self.frame_cache.remove(&blk_idx);
                }
                let step = min(end_idx - inaddr_idx, BLKS_PER_FRAME - offset);
                inaddr_idx += step;
                blk_idx += step;
            }
        }
        Ok(())
    }

    fn write_new_address(&mut self, id: &Eid, addr: &Addr) -> Result<()> {
        // if the old address exists, remove all of its blocks
        match self.get_address(id) {
            Ok(old_addr) => {
                self.remove_address_blocks(&old_addr)?;
            }
            Err(ref err) if *err == Error::NotFound => {}
            Err(err) => return Err(err),
        }

        // write new address
        self.put_address(id, addr)
    }

    pub fn del(&mut self, id: &Eid) -> Result<()> {
        // get address first
        let addr = match self.get_address(id) {
            Ok(addr) => addr,
            Err(ref err) if *err == Error::NotFound => return Ok(()),
            Err(err) => return Err(err),
        };

        // remove blocks in the address
        self.remove_address_blocks(&addr)?;

        // remove address
        self.depot.del_address(id)?;
        self.addr_cache.remove(id);

        Ok(())
    }
}

impl Debug for Storage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Storage")
            .field("depot", &self.depot)
            .field("allocator", &self.allocator)
            .finish()
    }
}

impl IntoRef for Storage {}

/// Storage reference type
pub type StorageRef = Arc<RwLock<Storage>>;
