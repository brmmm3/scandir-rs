use std::path::{Path, PathBuf};

#[cfg(feature = "bincode")]
use bincode_next::error::EncodeError;
#[cfg(feature = "speedy")]
use speedy::Writable;

use crate::{
    ErrorsType,
    direntry::{DirEntry, DirEntryExt},
};

#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub enum ScandirResult {
    DirEntry(DirEntry),
    DirEntryExt(DirEntryExt),
    Error((PathBuf, String)),
}

#[cfg(feature = "speedy")]
impl<C: speedy::Context> speedy::Writable<C> for ScandirResult {
    fn write_to<W: speedy::Writer<C> + ?Sized>(&self, writer: &mut W) -> Result<(), C::Error> {
        match self {
            Self::DirEntry(entry) => {
                writer.write_u32(0)?;
                writer.write_value(entry)
            }
            Self::DirEntryExt(entry) => {
                writer.write_u32(1)?;
                writer.write_value(entry)
            }
            Self::Error((path, message)) => {
                writer.write_u32(2)?;
                crate::speedy_path::write_path(path, writer)?;
                writer.write_value(message)
            }
        }
    }
}

#[cfg(feature = "speedy")]
impl<'a, C: speedy::Context> speedy::Readable<'a, C> for ScandirResult {
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        match reader.read_u32()? {
            0 => Ok(Self::DirEntry(reader.read_value()?)),
            1 => Ok(Self::DirEntryExt(reader.read_value()?)),
            2 => Ok(Self::Error((
                crate::speedy_path::read_path(reader)?,
                reader.read_value()?,
            ))),
            tag => Err(speedy::Error::custom(format!("Invalid ScandirResult tag: {tag}")).into()),
        }
    }
}

impl ScandirResult {
    #[inline]
    pub fn path(&self) -> &Path {
        match self {
            Self::DirEntry(e) => &e.path,
            Self::DirEntryExt(e) => &e.path,
            Self::Error(e) => &e.0,
        }
    }

    #[inline]
    pub fn error(&self) -> Option<&(PathBuf, String)> {
        match self {
            Self::Error(e) => Some(e),
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
            Self::DirEntryExt(e) => Some(e),
            _ => None,
        }
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.write_to_vec()
    }

    #[cfg(feature = "bincode")]
    pub fn to_bincode(&self) -> Result<Vec<u8>, EncodeError> {
        bincode_next::serde::encode_to_vec(self, bincode_next::config::legacy())
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct ScandirResults {
    pub results: Vec<ScandirResult>,
    pub errors: ErrorsType,
}

#[cfg(feature = "speedy")]
impl<C: speedy::Context> speedy::Writable<C> for ScandirResults {
    fn write_to<W: speedy::Writer<C> + ?Sized>(&self, writer: &mut W) -> Result<(), C::Error> {
        writer.write_value(&self.results)?;
        let errors = self
            .errors
            .iter()
            .map(|(path, message)| Ok((crate::speedy_path::path_str::<C>(path)?, message)))
            .collect::<Result<Vec<_>, C::Error>>()?;
        writer.write_value(&errors)
    }
}

#[cfg(feature = "speedy")]
impl<'a, C: speedy::Context> speedy::Readable<'a, C> for ScandirResults {
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let results = reader.read_value()?;
        let errors: Vec<(String, String)> = reader.read_value()?;
        Ok(Self {
            results,
            errors: errors
                .into_iter()
                .map(|(path, message)| (path.into(), message))
                .collect(),
        })
    }
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

    pub fn extend(&mut self, other: &ScandirResults) {
        self.results.extend_from_slice(&other.results);
        self.errors.extend_from_slice(&other.errors);
    }

    #[inline]
    pub fn push_entry(&mut self, entry: ScandirResult) {
        self.results.push(entry);
    }

    /// Return all directory entries, with optional filtering and no extra memory allocation.
    #[inline]
    pub fn dirs(&self) -> impl Iterator<Item = &ScandirResult> {
        self.results.iter().filter(|e| e.is_dir())
    }

    /// Return all file entries, with optional filtering and no extra memory allocation.
    #[inline]
    pub fn files(&self) -> impl Iterator<Item = &ScandirResult> {
        self.results.iter().filter(|e| e.is_file())
    }

    /// Return all symbolic link entries, with optional filtering and no extra memory allocation.
    #[inline]
    pub fn symlinks(&self) -> impl Iterator<Item = &ScandirResult> {
        self.results.iter().filter(|e| e.is_symlink())
    }

    /// Return all entries of other types, with optional filtering and no extra memory allocation.
    #[inline]
    pub fn other(&self) -> impl Iterator<Item = &ScandirResult> {
        self.results
            .iter()
            .filter(|e| !e.is_dir() && !e.is_file() && !e.is_symlink())
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.write_to_vec()
    }

    #[cfg(feature = "bincode")]
    pub fn to_bincode(&self) -> Result<Vec<u8>, EncodeError> {
        bincode_next::serde::encode_to_vec(self, bincode_next::config::legacy())
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
