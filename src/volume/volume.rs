use crate::util::crypto::{Cipher, Cost, Salt};
use crate::util::time::Time;
use crate::util::version::Version;
use crate::volume::storage::storage::StorageRef;
use crate::trans::eid::Eid;

/// Volume info
#[derive(Debug, Clone, Default)]
pub struct Info {
    pub id: Eid,
    pub ver: Version,
    pub uri: String,
    pub compress: bool,
    pub cost: Cost,
    pub cipher: Cipher,
    pub ctime: Time,
}

/// Volume
#[derive(Debug)]
pub struct Volume {
    info: Info,
    storage: StorageRef,
}

impl Volume {}
