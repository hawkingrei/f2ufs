#![feature(integer_atomics)]
pub mod f2fs;
pub mod trace;
pub type block_t = u32;
pub type nid_t = u32;

use std::sync::atomic::AtomicU8;

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
