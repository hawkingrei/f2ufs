pub mod file;
pub mod storage;

use std::fmt::Debug;

use crate::error::Result;
use crate::trans::Eid;
use crate::volume::address::Span;
use crate::util::crypto::Crypto;
use crate::util::crypto::Key;

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

    // close a storage
    fn close(&mut self) -> Result<()>;

    // super block operations
    fn get_super_block(&mut self, suffix: u64) -> Result<Vec<u8>>;
    fn put_super_block(&mut self, super_blk: &[u8], suffix: u64) -> Result<()>;

    // address operations
    fn get_address(&mut self, id: &Eid) -> Result<Vec<u8>>;
    fn put_address(&mut self, id: &Eid, addr: &[u8]) -> Result<()>;
    fn del_address(&mut self, id: &Eid) -> Result<()>;

    // block operations
    fn get_blocks(&mut self, dst: &mut [u8], span: Span) -> Result<()>;
    fn put_blocks(&mut self, span: Span, blks: &[u8]) -> Result<()>;
    fn del_blocks(&mut self, span: Span) -> Result<()>;

    // flush to storage
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}