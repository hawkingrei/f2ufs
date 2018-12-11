use std::fmt::{self, Debug};

use crate::trans::eid::Eid;
use crate::util::collections::HashMap;
use crate::volume::armor::Arm;  
use crate::trans::eid::Id;
use crate::util::lru::PinChecker;
use crate::util::lru::CountMeter; 
use crate::util::lru::Lru;
use crate::util::crypto::HashKey;
use crate::volume::storage::file::file_armor::FileArmor;

// entity index
#[derive(Clone, Deserialize, Serialize)]
struct Index {
    id: Eid,
    seq: u64,
    arm: Arm,
    map: HashMap<Eid, Vec<u8>>,
}

impl Index {
    fn new(id: Eid) -> Self {
        Index {
            id,
            seq: 0,
            arm: Arm::default(),
            map: HashMap::new(),
        }
    }
}

impl Id for Index {
    #[inline]
    fn id(&self) -> &Eid {
        &self.id
    }

    #[inline]
    fn id_mut(&mut self) -> &mut Eid {
        &mut self.id
    }
}


impl Seq for Index {
    #[inline]
    fn seq(&self) -> u64 {
        self.seq
    }

    #[inline]
    fn inc_seq(&mut self) {
        self.seq += 1
    }
}

impl<'de> ArmAccess<'de> for Index {
    #[inline]
    fn arm(&self) -> Arm {
        self.arm
    }

    #[inline]
    fn arm_mut(&mut self) -> &mut Arm {
        &mut self.arm
    }
}

impl Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Index")
            .field("id", &self.id)
            .field("seq", &self.seq)
            .field("arm", &self.arm)
            .field("map.len", &self.map.len())
            .finish()
    }
}

// entity index manager
pub struct IndexMgr {
    idx_armor: FileArmor<Index>,
    cache: Lru<u8, Index, CountMeter<Index>, PinChecker<Index>>,
    hash_key: HashKey,
}