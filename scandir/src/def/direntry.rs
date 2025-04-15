use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "bincode")]
use bincode::error::EncodeError;
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirEntry {
    pub path: String,
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    pub st_ctime: Option<SystemTime>,
    pub st_mtime: Option<SystemTime>,
    pub st_atime: Option<SystemTime>,
    pub st_size: u64,
}

impl DirEntry {
    #[inline]
    pub fn ctime(&self) -> f64 {
        let duration = self
            .st_ctime
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        (duration.as_secs() as f64) + (duration.subsec_nanos() as f64) * 1e-9
    }

    #[inline]
    pub fn mtime(&self) -> f64 {
        let duration = self
            .st_mtime
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        (duration.as_secs() as f64) + (duration.subsec_nanos() as f64) * 1e-9
    }

    #[inline]
    pub fn atime(&self) -> f64 {
        let duration = self
            .st_atime
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        (duration.as_secs() as f64) + (duration.subsec_nanos() as f64) * 1e-9
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.write_to_vec()
    }

    #[cfg(feature = "bincode")]
    pub fn to_vec(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::legacy())
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirEntryExt {
    pub path: String,
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    /// Creation time in seconds as float
    pub st_ctime: Option<SystemTime>,
    /// Modification time in seconds as float
    pub st_mtime: Option<SystemTime>,
    /// Access time in seconds as float
    pub st_atime: Option<SystemTime>,
    /// Size of file / entry
    pub st_size: u64,
    /// File system block size
    pub st_blksize: u64,
    /// Number of used blocks on device / file system
    pub st_blocks: u64,
    /// File access mode / rights
    pub st_mode: u32,
    /// Number of hardlinks
    pub st_nlink: u64,
    /// User ID (Unix only)
    pub st_uid: u32,
    /// Group ID (Unix only)
    pub st_gid: u32,
    /// I-Node number (Unix only)
    pub st_ino: u64,
    /// Device number (Unix only)
    pub st_dev: u64,
    /// Device number (for character and block devices on Unix).
    pub st_rdev: u64,
}

impl DirEntryExt {
    #[inline]
    pub fn ctime(&self) -> f64 {
        let duration = self
            .st_ctime
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        (duration.as_secs() as f64) + (duration.subsec_nanos() as f64) * 1e-9
    }

    #[inline]
    pub fn mtime(&self) -> f64 {
        let duration = self
            .st_mtime
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        (duration.as_secs() as f64) + (duration.subsec_nanos() as f64) * 1e-9
    }

    #[inline]
    pub fn atime(&self) -> f64 {
        let duration = self
            .st_atime
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        (duration.as_secs() as f64) + (duration.subsec_nanos() as f64) * 1e-9
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.write_to_vec()
    }

    #[cfg(feature = "bincode")]
    pub fn to_vec(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::legacy())
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}
