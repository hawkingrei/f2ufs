pub mod eid;

pub use self::eid::{Eid, Id};

use std::io::Write;

use crate::error::Result;

/// Finish trait, used with writer which implements std::io::Write trait
pub trait Finish: Write {
    fn finish(self) -> Result<()>;
    fn finish_and_flush(self) -> Result<()>;
}
