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
extern crate env_logger;
extern crate rmp_serde;

macro_rules! map_io_err {
    ($x:expr) => {
        $x.map_err(|e| IoError::new(ErrorKind::Other, e.description()));
    };
}

pub mod content;
pub mod diskptr;
pub mod error;
pub mod file;
pub mod fs;
pub mod repo;
pub mod trans;
pub mod util;
pub mod version;
pub mod volume;

// block and frame size
pub const BLK_SIZE: usize = 8 * 1024;
pub const BLKS_PER_FRAME: usize = 16;
pub const FRAME_SIZE: usize = BLKS_PER_FRAME * BLK_SIZE;

pub use self::error::{Error, Result};

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
