use std::path::{Path, PathBuf};

#[cfg(feature = "bincode")]
use bincode_next::error::EncodeError;
#[cfg(feature = "speedy")]
use speedy::Writable;

#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct Toc {
    pub dirs: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
    pub symlinks: Vec<PathBuf>,
    pub other: Vec<PathBuf>,
    pub errors: Vec<String>,
}

#[cfg(feature = "speedy")]
impl<C: speedy::Context> speedy::Writable<C> for Toc {
    fn write_to<W: speedy::Writer<C> + ?Sized>(&self, writer: &mut W) -> Result<(), C::Error> {
        crate::speedy_path::write_paths(&self.dirs, writer)?;
        crate::speedy_path::write_paths(&self.files, writer)?;
        crate::speedy_path::write_paths(&self.symlinks, writer)?;
        crate::speedy_path::write_paths(&self.other, writer)?;
        writer.write_value(&self.errors)
    }
}

#[cfg(feature = "speedy")]
impl<'a, C: speedy::Context> speedy::Readable<'a, C> for Toc {
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        Ok(Self {
            dirs: crate::speedy_path::read_paths(reader)?,
            files: crate::speedy_path::read_paths(reader)?,
            symlinks: crate::speedy_path::read_paths(reader)?,
            other: crate::speedy_path::read_paths(reader)?,
            errors: reader.read_value()?,
        })
    }
}

impl Toc {
    pub fn new() -> Self {
        Toc {
            dirs: Vec::new(),
            files: Vec::new(),
            symlinks: Vec::new(),
            other: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Construct from an owned Toc by moving it, so the binding layer can avoid a deep copy.
    pub fn from_owned(toc: Toc) -> Self {
        toc
    }

    pub fn clear(&mut self) {
        self.dirs.clear();
        self.files.clear();
        self.symlinks.clear();
        self.other.clear();
        self.errors.clear();
    }

    pub fn dirs(&self) -> Vec<PathBuf> {
        self.dirs.clone()
    }

    pub fn files(&self) -> Vec<PathBuf> {
        self.files.clone()
    }

    pub fn symlinks(&self) -> Vec<PathBuf> {
        self.symlinks.clone()
    }

    pub fn other(&self) -> Vec<PathBuf> {
        self.other.clone()
    }

    pub fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.dirs.is_empty()
            && self.files.is_empty()
            && self.symlinks.is_empty()
            && self.other.is_empty()
            && self.errors.is_empty()
    }

    pub fn extend(&mut self, root_dir: &Path, other: &Toc) {
        // When the prefix is empty, skip the unnecessary string concatenation and clone directly.
        if root_dir.as_os_str().is_empty() {
            self.dirs.extend(other.dirs.iter().cloned());
            self.files.extend(other.files.iter().cloned());
            self.symlinks.extend(other.symlinks.iter().cloned());
            self.other.extend(other.other.iter().cloned());
            self.errors.extend(other.errors.iter().cloned());
            return;
        }
        self.dirs.reserve(other.dirs.len());
        self.dirs
            .extend(other.dirs.iter().map(|x| root_dir.join(x)));
        self.files.reserve(other.files.len());
        self.files
            .extend(other.files.iter().map(|x| root_dir.join(x)));
        self.symlinks.reserve(other.symlinks.len());
        self.symlinks
            .extend(other.symlinks.iter().map(|x| root_dir.join(x)));
        self.other.reserve(other.other.len());
        self.other
            .extend(other.other.iter().map(|x| root_dir.join(x)));
        self.errors.reserve(other.errors.len());
        self.errors.extend(other.errors.iter().map(|error| {
            if let Some(root) = root_dir.to_str() {
                format!("{root}/{error}")
            } else {
                format!("{root_dir:?}/{error}")
            }
        }));
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

impl Default for Toc {
    fn default() -> Self {
        Self::new()
    }
}
