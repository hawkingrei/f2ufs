
use std::path::PathBuf;

/// File Storage
#[derive(Debug)]
pub struct FileStorage {
    base: PathBuf,
    idx_mgr: IndexMgr,
    sec_mgr: SectorMgr,
}