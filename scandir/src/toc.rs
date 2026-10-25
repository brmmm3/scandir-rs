#[cfg(feature = "bincode")]
use bincode_next::error::EncodeError;
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct Toc {
    pub dirs: Vec<String>,
    pub files: Vec<String>,
    pub symlinks: Vec<String>,
    pub other: Vec<String>,
    pub errors: Vec<String>,
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

    pub fn dirs(&self) -> Vec<String> {
        self.dirs.clone()
    }

    pub fn files(&self) -> Vec<String> {
        self.files.clone()
    }

    pub fn symlinks(&self) -> Vec<String> {
        self.symlinks.clone()
    }

    pub fn other(&self) -> Vec<String> {
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

    pub fn extend(&mut self, root_dir: &str, other: &Toc) {
        // When the prefix is empty, skip the unnecessary string concatenation and clone directly.
        if root_dir.is_empty() {
            self.dirs.extend(other.dirs.iter().cloned());
            self.files.extend(other.files.iter().cloned());
            self.symlinks.extend(other.symlinks.iter().cloned());
            self.other.extend(other.other.iter().cloned());
            self.errors.extend(other.errors.iter().cloned());
            return;
        }
        let root_len = root_dir.len();
        let join_path = |x: &str| -> String {
            let mut path = String::with_capacity(root_len + 1 + x.len());
            path.push_str(root_dir);
            path.push('/');
            path.push_str(x);
            path
        };
        self.dirs.reserve(other.dirs.len());
        self.dirs.extend(other.dirs.iter().map(|x| join_path(x)));
        self.files.reserve(other.files.len());
        self.files.extend(other.files.iter().map(|x| join_path(x)));
        self.symlinks.reserve(other.symlinks.len());
        self.symlinks
            .extend(other.symlinks.iter().map(|x| join_path(x)));
        self.other.reserve(other.other.len());
        self.other.extend(other.other.iter().map(|x| join_path(x)));
        self.errors.reserve(other.errors.len());
        self.errors
            .extend(other.errors.iter().map(|x| join_path(x)));
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
