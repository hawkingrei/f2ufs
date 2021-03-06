mod file;
mod mem;
pub mod storage;

pub use self::file::FileStorage;
pub use self::mem::MemStorage;
pub use self::storage::{Reader, Storage, StorageRef, Writer};

use std::fmt::Debug;

use crate::error::Result;
use crate::trans::Eid;
use crate::util::crypto::{Crypto, Key};
use crate::volume::address::Span;
/// Storable trait
pub trait Storable: Debug + Send + Sync {
    // check if storage exists
    fn exists(&self) -> Result<bool>;

    // make connection to storage
    fn connect(&mut self) -> Result<()>;

    // initial a storage
    fn init(&mut self, crypto: Crypto, key: Key) -> Result<()>;

    // open a storage
    fn open(&mut self, crypto: Crypto, key: Key) -> Result<()>;

    // super block read/write, must not buffered
    // write no need to be atomic, but must gurantee any successful
    // write is persistent
    fn get_super_block(&mut self, suffix: u64) -> Result<Vec<u8>>;
    fn put_super_block(&mut self, super_blk: &[u8], suffix: u64) -> Result<()>;

    // wal read/write, must not buffered
    // update no need to be atomic, but must gurantee any successful
    // update is persistent
    fn get_wal(&mut self, id: &Eid) -> Result<Vec<u8>>;
    fn put_wal(&mut self, id: &Eid, wal: &[u8]) -> Result<()>;
    fn del_wal(&mut self, id: &Eid) -> Result<()>;

    // address read/write, can be buffered
    // storage doesn't need to gurantee update is persistent
    fn get_address(&mut self, id: &Eid) -> Result<Vec<u8>>;
    fn put_address(&mut self, id: &Eid, addr: &[u8]) -> Result<()>;
    fn del_address(&mut self, id: &Eid) -> Result<()>;

    // block read/write, can be buffered
    // storage doesn't need to gurantee update is persistent
    fn get_blocks(&mut self, dst: &mut [u8], span: Span) -> Result<()>;
    fn put_blocks(&mut self, span: Span, blks: &[u8]) -> Result<()>;
    fn del_blocks(&mut self, span: Span) -> Result<()>;

    // flush possibly buffered address and block to storage
    // storage must gurantee write is persistent
    fn flush(&mut self) -> Result<()>;
}
