use std::sync::atomic::AtomicU8;
struct f2fs_inode_info {
    /* Use below internally in f2fs*/
    dirty_pages: AtomicU8,
}
