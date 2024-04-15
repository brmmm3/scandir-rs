use speedy::{ Readable, Writable };

use crate::ErrorsType;
use crate::direntry::{ DirEntry, DirEntryExt };

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(any(feature = "bincode", feature = "json"), derive(Deserialize, Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ScandirResult {
    DirEntry(DirEntry),
    DirEntryExt(DirEntryExt),
    Error((String, String)),
}

impl ScandirResult {
    #[inline]
    pub fn path(&self) -> &String {
        match self {
            Self::DirEntry(e) => &e.path,
            Self::DirEntryExt(e) => &e.path,
            Self::Error(e) => &e.0,
        }
    }

    #[inline]
    pub fn error(&self) -> Option<&(String, String)> {
        match self {
            Self::Error(ref e) => Some(e),
            _ => None,
        }
    }

    #[inline]
    pub fn is_dir(&self) -> bool {
        match self {
            Self::DirEntry(e) => e.is_dir,
            Self::DirEntryExt(e) => e.is_dir,
            Self::Error(_) => false,
        }
    }

    #[inline]
    pub fn is_file(&self) -> bool {
        match self {
            Self::DirEntry(e) => e.is_file,
            Self::DirEntryExt(e) => e.is_file,
            Self::Error(_) => false,
        }
    }

    #[inline]
    pub fn is_symlink(&self) -> bool {
        match self {
            Self::DirEntry(e) => e.is_symlink,
            Self::DirEntryExt(e) => e.is_symlink,
            Self::Error(_) => false,
        }
    }

    #[inline]
    pub fn ctime(&self) -> f64 {
        match self {
            Self::DirEntry(e) => e.ctime(),
            Self::DirEntryExt(e) => e.ctime(),
            Self::Error(_) => 0.0,
        }
    }

    #[inline]
    pub fn mtime(&self) -> f64 {
        match self {
            Self::DirEntry(e) => e.mtime(),
            Self::DirEntryExt(e) => e.mtime(),
            Self::Error(_) => 0.0,
        }
    }

    #[inline]
    pub fn atime(&self) -> f64 {
        match self {
            Self::DirEntry(e) => e.atime(),
            Self::DirEntryExt(e) => e.atime(),
            Self::Error(_) => 0.0,
        }
    }

    #[inline]
    pub fn size(&self) -> u64 {
        match self {
            Self::DirEntry(e) => e.st_size,
            Self::DirEntryExt(e) => e.st_size,
            Self::Error(_) => 0,
        }
    }

    #[inline]
    pub fn ext(&self) -> Option<&DirEntryExt> {
        match self {
            Self::DirEntryExt(ref e) => Some(e),
            _ => None,
        }
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.write_to_vec()
    }

    #[cfg(feature = "bincode")]
    pub fn to_bincode(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self)
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(any(feature = "bincode", feature = "json"), derive(Deserialize, Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ScandirResults {
    pub results: Vec<ScandirResult>,
    pub errors: ErrorsType,
}

impl ScandirResults {
    pub fn new() -> Self {
        ScandirResults {
            results: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.results.clear();
        self.errors.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty() && self.errors.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.results.len() + self.errors.len()
    }

    pub fn extend(&mut self, results: &ScandirResults) {
        self.results.extend_from_slice(&results.results);
        self.errors.extend_from_slice(&results.errors);
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.write_to_vec()
    }

    #[cfg(feature = "bincode")]
    pub fn to_bincode(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self)
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

impl Default for ScandirResults {
    fn default() -> Self {
        Self::new()
    }
}
