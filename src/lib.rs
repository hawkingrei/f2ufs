#![feature(integer_atomics)]
pub mod f2fs;
pub mod trace;
pub mod diskptr;
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
