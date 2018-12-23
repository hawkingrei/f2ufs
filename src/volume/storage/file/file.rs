use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::path::PathBuf;

use super::vio;
use crate::error::{Error, Result};
use crate::trans::eid::Eid;
use crate::util;
use crate::util::crypto::Crypto;
use crate::util::crypto::Key;
use crate::volume::address::Span;
use crate::volume::storage::file::index::IndexMgr;
use crate::volume::storage::file::sector::SectorMgr;
use crate::volume::storage::Storable;

/// File Storage
#[derive(Debug)]
pub struct FileStorage {
    base: PathBuf,
    wal_base: PathBuf,
    idx_mgr: IndexMgr,
    sec_mgr: SectorMgr,
}

impl FileStorage {
    // super block file name
    const SUPER_BLK_FILE_NAME: &'static str = "super_blk";

    // wal, index and data dir names
    const WAL_DIR: &'static str = "wal";
    const INDEX_DIR: &'static str = "index";
    const DATA_DIR: &'static str = "data";

    // index and data subkey ids
    const SUBKEY_ID_INDEX: u64 = 42;
    const SUBKEY_ID_SECTOR: u64 = 43;

    pub fn new(base: &Path) -> Self {
        FileStorage {
            base: base.to_path_buf(),
            wal_base: base.join(Self::WAL_DIR),
            idx_mgr: IndexMgr::new(&base.join(Self::INDEX_DIR)),
            sec_mgr: SectorMgr::new(&base.join(Self::DATA_DIR)),
        }
    }

    #[inline]
    fn super_block_path(&self, suffix: u64) -> PathBuf {
        let mut path = self.base.join(Self::SUPER_BLK_FILE_NAME);
        path.set_extension(format!("{}", suffix));
        path
    }

    // wal file path
    #[inline]
    fn wal_path(&self, id: &Eid) -> PathBuf {
        id.to_path_buf(&self.wal_base)
    }

    #[inline]
    fn index_dir(&self) -> PathBuf {
        self.base.join(Self::INDEX_DIR)
    }

    #[inline]
    fn data_dir(&self) -> PathBuf {
        self.base.join(Self::DATA_DIR)
    }

    fn set_crypto_ctx(&mut self, crypto: Crypto, key: Key) {
        let hash_key = key.derive(Self::SUBKEY_ID_INDEX);
        self.idx_mgr
            .set_crypto_ctx(crypto.clone(), key.clone(), hash_key);
        let hash_key = key.derive(Self::SUBKEY_ID_SECTOR);
        self.sec_mgr
            .set_crypto_ctx(crypto.clone(), key.clone(), hash_key);
    }
}

impl Storable for FileStorage {
    #[inline]
    fn exists(&self) -> Result<bool> {
        match vio::metadata(&self.base) {
            Ok(_) => Ok(true),
            Err(ref err) if err.kind() == ErrorKind::NotFound => Ok(false),
            Err(err) => Err(Error::from(err)),
        }
    }

    #[inline]
    fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    fn init(&mut self, crypto: Crypto, key: Key) -> Result<()> {
        // create dir structure
        vio::create_dir_all(self.index_dir())?;
        vio::create_dir_all(self.data_dir())?;

        // set crypto context
        self.set_crypto_ctx(crypto, key);

        Ok(())
    }

    #[inline]
    fn open(&mut self, crypto: Crypto, key: Key) -> Result<()> {
        self.set_crypto_ctx(crypto, key);
        Ok(())
    }

    fn get_super_block(&mut self, suffix: u64) -> Result<Vec<u8>> {
        let path = self.super_block_path(suffix);
        let mut buf = Vec::new();
        let mut file = vio::OpenOptions::new().read(true).open(&path)?;
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn put_super_block(&mut self, super_blk: &[u8], suffix: u64) -> Result<()> {
        let path = self.super_block_path(suffix);
        let mut file = vio::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(super_blk)?;
        Ok(())
    }

    fn get_wal(&mut self, id: &Eid) -> Result<Vec<u8>> {
        let path = self.wal_path(id);
        if !path.exists() {
            return Err(Error::NotFound);
        }

        let mut ret = Vec::new();
        let mut file = vio::OpenOptions::new().read(true).open(&path)?;
        file.read_to_end(&mut ret)?;

        Ok(ret)
    }

    fn put_wal(&mut self, id: &Eid, wal: &[u8]) -> Result<()> {
        let path = self.wal_path(id);
        util::ensure_parents_dir(&path)?;
        let mut file = vio::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(wal)?;
        Ok(())
    }

    fn del_wal(&mut self, id: &Eid) -> Result<()> {
        let path = self.wal_path(id);
        if path.exists() {
            vio::remove_file(&path)?;
            util::remove_empty_parent_dir(&path)?;
        }
        Ok(())
    }

    #[inline]
    fn get_address(&mut self, id: &Eid) -> Result<Vec<u8>> {
        self.idx_mgr.read_addr(id)
    }

    #[inline]
    fn put_address(&mut self, id: &Eid, addr: &[u8]) -> Result<()> {
        self.idx_mgr.write_addr(id, addr)
    }

    #[inline]
    fn del_address(&mut self, id: &Eid) -> Result<()> {
        self.idx_mgr.del_address(id)
    }

    #[inline]
    fn get_blocks(&mut self, dst: &mut [u8], span: Span) -> Result<()> {
        self.sec_mgr.read_blocks(dst, span)
    }

    #[inline]
    fn put_blocks(&mut self, span: Span, blks: &[u8]) -> Result<()> {
        self.sec_mgr.write_blocks(span, blks)
    }

    #[inline]
    fn del_blocks(&mut self, span: Span) -> Result<()> {
        self.sec_mgr.del_blocks(span)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        self.idx_mgr.flush()
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::fs;
    use std::time::Instant;

    use self::tempdir::TempDir;
    use super::*;
    use crate::error::Error;
    use crate::util::crypto::{Crypto, RandomSeed, RANDOM_SEED_SIZE};
    use crate::util::init_env;
    use crate::util::speed_str;
    use crate::BLK_SIZE;

    fn setup() -> (PathBuf, TempDir) {
        init_env();
        let tmpdir = TempDir::new("zbox_test").expect("Create temp dir failed");
        let dir = tmpdir.path().to_path_buf();
        //let dir = PathBuf::from("./tt");
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        (dir, tmpdir)
    }

    #[test]
    fn super_blk_oper() {
        let (dir, _tmpdir) = setup();
        let blk = vec![1, 2, 3];
        let blk2 = vec![4, 5, 6];
        let mut fs = FileStorage::new(&dir);
        fs.init(Crypto::default(), Key::new_empty()).unwrap();

        // put super block
        fs.put_super_block(&blk, 0).unwrap();
        fs.put_super_block(&blk2, 1).unwrap();

        // get super block
        let tgt = fs.get_super_block(0).unwrap();
        assert_eq!(&tgt[..], &blk[..]);
        let tgt = fs.get_super_block(1).unwrap();
        assert_eq!(&tgt[..], &blk2[..]);
    }

    #[test]
    fn wal_oper() {
        let (dir, _tmpdir) = setup();
        let mut fs = FileStorage::new(&dir);
        fs.init(Crypto::default(), Key::new_empty()).unwrap();

        let id = Eid::new();
        let id2 = Eid::new();
        let wal = vec![1, 2, 3];
        let wal2 = vec![4, 5, 6];

        // add wal 1
        fs.put_wal(&id, &wal).unwrap();
        let tgt = fs.get_wal(&id).unwrap();
        assert_eq!(&tgt[..], &wal[..]);

        // add wal 2
        fs.put_wal(&id2, &wal2).unwrap();
        let tgt = fs.get_wal(&id2).unwrap();
        assert_eq!(&tgt[..], &wal2[..]);

        // delete wal 1, wal 2 should still be there
        fs.del_wal(&id).unwrap();
        assert_eq!(fs.get_wal(&id).unwrap_err(), Error::NotFound);
        let tgt = fs.get_wal(&id2).unwrap();
        assert_eq!(&tgt[..], &wal2[..]);

        // re-open storage
        drop(fs);
        let mut fs = FileStorage::new(&dir);
        fs.open(Crypto::default(), Key::new_empty()).unwrap();

        // wal 1 is deleted, wal 2 should still be there
        assert_eq!(fs.get_wal(&id).unwrap_err(), Error::NotFound);
        let tgt = fs.get_wal(&id2).unwrap();
        assert_eq!(&tgt[..], &wal2[..]);
    }

    #[test]
    fn index_oper() {
        let (dir, _tmpdir) = setup();
        let mut fs = FileStorage::new(&dir);
        fs.init(Crypto::default(), Key::new_empty()).unwrap();

        let id = Eid::new();
        let id2 = Eid::new();
        let addr = vec![1, 2, 3];
        let addr2 = vec![4, 5, 6];

        // add address 1
        fs.put_address(&id, &addr).unwrap();
        let tgt = fs.get_address(&id).unwrap();
        assert_eq!(&tgt[..], &addr[..]);

        // add address 2
        fs.put_address(&id2, &addr2).unwrap();
        let tgt = fs.get_address(&id2).unwrap();
        assert_eq!(&tgt[..], &addr2[..]);

        // delete address 1, address 2 should still be there
        fs.del_address(&id).unwrap();
        assert_eq!(fs.get_address(&id).unwrap_err(), Error::NotFound);
        let tgt = fs.get_address(&id2).unwrap();
        assert_eq!(&tgt[..], &addr2[..]);

        fs.flush().unwrap();

        // re-open storage
        drop(fs);
        let mut fs = FileStorage::new(&dir);
        fs.open(Crypto::default(), Key::new_empty()).unwrap();

        // address 1 is deleted, address 2 should still be there
        assert_eq!(fs.get_address(&id).unwrap_err(), Error::NotFound);
        let tgt = fs.get_address(&id2).unwrap();
        assert_eq!(&tgt[..], &addr2[..]);
    }

    #[test]
    fn block_oper() {
        let (dir, _tmpdir) = setup();
        let mut fs = FileStorage::new(&dir);
        fs.init(Crypto::default(), Key::new_empty()).unwrap();

        let mut blks = vec![1u8; BLK_SIZE * 4];
        blks[0] = 42u8;
        blks[BLK_SIZE] = 43u8;
        blks[BLK_SIZE * 2] = 44u8;
        blks[BLK_SIZE * 3] = 45u8;
        blks[BLK_SIZE * 4 - 1] = 46u8;
        let mut tgt = vec![0u8; BLK_SIZE * 4];

        // write 4 blocks
        fs.put_blocks(Span::new(0, 4), &blks).unwrap();

        // read 4 blocks
        fs.get_blocks(&mut tgt, Span::new(0, 4)).unwrap();
        assert_eq!(&tgt[..], &blks[..]);

        // delete block 1, block 2 should still be there
        {
            let blk = &mut tgt[..BLK_SIZE];
            fs.del_blocks(Span::new(1, 1)).unwrap();
            assert_eq!(
                fs.get_blocks(blk, Span::new(1, 1)).unwrap_err(),
                Error::NotFound
            );
            fs.get_blocks(blk, Span::new(2, 1)).unwrap();
            assert_eq!(blk, &blks[BLK_SIZE * 2..BLK_SIZE * 3]);
        }

        // get continuous blocks with deleted block inside should fail
        assert_eq!(
            fs.get_blocks(&mut tgt, Span::new(0, 4)).unwrap_err(),
            Error::NotFound
        );

        // write more blocks, more than a sector
        // sector #1: 4096 blocks, sector #2: 4 blocks
        let idx = 4;
        for i in 0..4096 / 4 {
            fs.put_blocks(Span::new(idx + i * 4, 4), &blks).unwrap();
        }

        // re-open storage
        drop(fs);
        let mut fs = FileStorage::new(&dir);
        fs.open(Crypto::default(), Key::new_empty()).unwrap();

        // blocks should still be there
        let blk = &mut tgt[..BLK_SIZE];
        fs.get_blocks(blk, Span::new(0, 1)).unwrap();
        assert_eq!(blk, &blks[..BLK_SIZE]);
        assert_eq!(
            fs.get_blocks(blk, Span::new(1, 1)).unwrap_err(),
            Error::NotFound
        );

        // delete many blocks in sector #1 should shrink the sector
        fs.del_blocks(Span::new(0, 4092)).unwrap();

        // delete all blocks in sector #1 should remove the sector
        fs.del_blocks(Span::new(0, 4096)).unwrap();

        // delete all blocks in unfiished sector #2 should not remove the sector
        fs.del_blocks(Span::new(4096, 4)).unwrap();

        // continu write until the end of sector #2,
        // this should shrink sector #2
        let idx = 4100;
        for i in 0..4092 / 4 {
            fs.del_blocks(Span::new(idx - 4 + i * 4, 4)).unwrap();
            fs.put_blocks(Span::new(idx + i * 4, 4), &blks).unwrap();
        }
    }

    #[test]
    fn test_perf() {
        let (dir, _tmpdir) = setup();
        let mut fs = FileStorage::new(&dir);
        fs.init(Crypto::default(), Key::new_empty()).unwrap();

        const DATA_LEN: usize = 36 * 1024 * 1024;
        const BLK_CNT: usize = DATA_LEN / BLK_SIZE;
        let mut buf = vec![0u8; DATA_LEN];
        let seed = RandomSeed::from(&[0u8; RANDOM_SEED_SIZE]);
        Crypto::random_buf_deterministic(&mut buf, &seed);
        let span = Span::new(0, BLK_CNT);

        // write
        let now = Instant::now();
        fs.put_blocks(span, &buf).unwrap();
        let write_time = now.elapsed();

        // read
        let now = Instant::now();
        fs.get_blocks(&mut buf, span).unwrap();
        let read_time = now.elapsed();

        println!(
            "File storage (depot) perf: read: {}, write: {}",
            speed_str(&read_time, DATA_LEN),
            speed_str(&write_time, DATA_LEN)
        );
    }
}
