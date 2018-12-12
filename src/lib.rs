#![feature(integer_atomics)]
#![feature(arbitrary_self_types, unsized_locals)]

#[macro_use]
extern crate log;
extern crate hashbrown;
extern crate indexmap;
extern crate linked_hash_map;
extern crate lz4;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bytes;
extern crate rmp_serde;

macro_rules! map_io_err {
    ($x:expr) => {
        $x.map_err(|e| IoError::new(ErrorKind::Other, e.description()));
    };
}

pub mod diskptr;
pub mod error;
pub mod f2fs;
pub mod fs;
pub mod inode;
pub mod mem_inode;
pub mod parallel_io;
pub mod segment;
pub mod trace;
pub mod trans;
pub mod util;
pub mod version;
pub mod volume;

use std::io;
use std::sync::atomic::AtomicU8;

// block and frame size
pub const BLK_SIZE: usize = 8 * 1024;
pub const BLKS_PER_FRAME: usize = 16;
pub const FRAME_SIZE: usize = BLKS_PER_FRAME * BLK_SIZE;

pub use self::error::{Error, Result};

pub type block_t = u32;
pub type nid_t = u32;

/// An offset for a storage file segment.
pub type SegmentId = usize;

/// A log file offset.
pub type LogId = u64;

/// A pointer to an blob blob.
pub type BlobPointer = Lsn;

/// A logical sequence number.
pub type Lsn = i64;

/// A page identifier.
pub type PageId = usize;

enum f2fs_fault {
    FAULT_KMALLOC,
    FAULT_KVMALLOC,
    FAULT_PAGE_ALLOC,
    FAULT_PAGE_GET,
    FAULT_ALLOC_BIO,
    FAULT_ALLOC_NID,
    FAULT_ORPHAN,
    FAULT_BLOCK,
    FAULT_DIR_DEPTH,
    FAULT_EVICT_INODE,
    FAULT_TRUNCATE,
    FAULT_IO,
    FAULT_CHECKPOINT,
    FAULT_DISCARD,
    FAULT_MAX,
}

struct f2fs_fault_info {
    inject_ops: AtomicU8,
    inject_rate: u32,
    inject_type: u32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
