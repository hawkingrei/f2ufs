use super::*;

pub enum DiskPtr {
    /// Points to a value stored in the single-file log.
    Inline(LogId),
    /// Points to a value stored off-log in the blob directory.
    Blob(LogId, BlobPointer),
}
