pub mod fnode;
pub mod fs;

use crate::content::store::StoreRef;
use crate::fs::fnode::FnodeRef;
use crate::fs::fs::ShutterRef;
use crate::trans::txmgr::TxMgrRef;
use crate::util::crypto::{Cipher, Cost, Crypto};
use crate::volume::volume::VolumeRef;

// Default file versoin limit
const DEFAULT_VERSION_LIMIT: u8 = 10;

// Options
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Options {
    pub version_limit: u8,
    pub dedup_chunk: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            version_limit: DEFAULT_VERSION_LIMIT,
            dedup_chunk: true,
        }
    }
}

// Configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub cost: Cost,
    pub cipher: Cipher,
    pub compress: bool,
    pub opts: Options,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cost: Cost::default(),
            cipher: if Crypto::is_aes_hardware_available() {
                Cipher::Aes
            } else {
                Cipher::Xchacha
            },
            compress: false,
            opts: Options::default(),
        }
    }
}

/// Open File Handle
#[derive(Debug, Clone)]
pub struct Handle {
    pub fnode: FnodeRef,
    pub store: StoreRef,
    pub txmgr: TxMgrRef,
    pub vol: VolumeRef,
    pub shutter: ShutterRef,
}
