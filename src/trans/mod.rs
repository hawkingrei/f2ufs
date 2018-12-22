pub mod cow;
pub mod eid;
pub mod trans;
mod txid;
pub mod txmgr;
mod wal;

pub use self::eid::{Eid, Id};
pub use self::txid::Txid;
pub use self::txmgr::{TxHandle, TxMgr, TxMgrRef};
pub use self::wal::EntityType;

use std::io::Write;

use crate::error::Result;

/// Finish trait, used with writer which implements std::io::Write trait
pub trait Finish: Write {
    fn finish(self) -> Result<()>;
}
