#![feature(integer_atomics)]
pub mod f2fs;
pub mod trace;
pub mod diskptr;

use std::sync::atomic::AtomicU8;

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
