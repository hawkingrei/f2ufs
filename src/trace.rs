use crate::block_t;

enum file_type {
    __NORMAL_FILE,
    __DIR_FILE,
    __NODE_FILE,
    __META_FILE,
    __ATOMIC_FILE,
    __VOLATILE_FILE,
    __MISC_FILE,
}

struct last_io_info {
    major: u8,
    minor: u8,
    pid: u32,
    ftype: file_type,
    len: block_t,
}
