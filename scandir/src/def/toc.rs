use std::path::PathBuf;

use speedy::{ Readable, Writable };

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(any(feature = "bincode", feature = "json"), derive(Deserialize, Serialize))]
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
        self.dirs.is_empty() &&
            self.files.is_empty() &&
            self.symlinks.is_empty() &&
            self.other.is_empty() &&
            self.errors.is_empty()
    }

    pub fn extend(&mut self, root_dir: &str, other: &Toc) {
        self.dirs.extend_from_slice(
            &other.dirs
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>()
        );
        self.files.extend_from_slice(
            &other.files
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>()
        );
        self.symlinks.extend_from_slice(
            &other.symlinks
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>()
        );
        self.other.extend_from_slice(
            &other.other
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>()
        );
        self.errors.extend_from_slice(
            &other.errors
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>()
        );
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

impl Default for Toc {
    fn default() -> Self {
        Self::new()
    }
}
