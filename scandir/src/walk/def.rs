use std::path::PathBuf;

use crate::Toc;

#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct WalkEntry {
    pub path: PathBuf,
    pub toc: Toc,
}

#[cfg(feature = "speedy")]
crate::speedy_path::impl_speedy_path_struct!(WalkEntry, path, toc);

#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct WalkEntryExt {
    pub path: PathBuf,
    pub toc: Toc,
}

#[cfg(feature = "speedy")]
crate::speedy_path::impl_speedy_path_struct!(WalkEntryExt, path, toc);

#[derive(Debug, Clone)]
pub enum WalkResult {
    Toc(Toc),
    WalkEntry(WalkEntry),
    WalkEntryExt(WalkEntryExt),
}
