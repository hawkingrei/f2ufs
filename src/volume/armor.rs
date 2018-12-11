use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::trans::eid::Eid;
use crate::util::crypto::Crypto;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum Arm {
    Left = 0,
    Right = 1,
}

impl Arm {
    fn to_eid(&self, id: &Eid) -> Eid {
        // serialize arm
        let mut arm_buf = Vec::new();
        self.serialize(&mut Serializer::new(&mut arm_buf)).unwrap();

        // hash eid and arm to make an eid
        let mut buf = Vec::new();
        buf.put(id.as_ref());
        buf.put(&arm_buf);
        let hash = Crypto::hash(&buf);
        Eid::from_slice(&hash)
    }

    #[inline]
    fn to_both_eid(id: &Eid) -> (Eid, Eid) {
        (Arm::Left.to_eid(id), Arm::Right.to_eid(id))
    }

    #[inline]
    pub fn other(&self) -> Self {
        match *self {
            Arm::Left => Arm::Right,
            Arm::Right => Arm::Left,
        }
    }

    #[inline]
    pub fn toggle(&mut self) {
        *self = self.other();
    }

    pub fn remove_arm(&self, id: &Eid, vol: &VolumeRef) -> Result<()> {
        let mut vol = vol.write().unwrap();
        let arm_id = self.to_eid(id);
        vol.del(&arm_id)
    }

    pub fn remove_all(id: &Eid, vol: &VolumeRef) -> Result<()> {
        let mut vol = vol.write().unwrap();
        let (left_arm_id, right_arm_id) = Arm::to_both_eid(id);
        vol.del(&left_arm_id).and(vol.del(&right_arm_id))
    }
}

impl Default for Arm {
    #[inline]
    fn default() -> Self {
        Arm::Left
    }
}
