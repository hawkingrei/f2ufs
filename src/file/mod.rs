use crate::error::Result;
use std::fmt::Debug;

/// Storable trait
pub trait Storable: Debug + Send + Sync {
    // check if storage exists
    fn exists(&self) -> Result<bool>;

    // make connection to storage
    fn connect(&mut self) -> Result<()>;

    // initial a storage
    fn init(&mut self) -> Result<()>;

    // open a storage
    fn open(&mut self) -> Result<()>;

    // close a storage
    fn close(&mut self) -> Result<()>;

    // super block operations
    fn get_super_block(&mut self, suffix: u64) -> Result<Vec<u8>>;
    fn put_super_block(&mut self, super_blk: &[u8], suffix: u64) -> Result<()>;

    // flush to storage
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
