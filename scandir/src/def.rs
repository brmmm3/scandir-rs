use std::path::PathBuf;

use glob_sl::{MatchOptions, Pattern};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

#[derive(Debug, Clone)]
pub struct Options {
    pub root_path: PathBuf,
    pub sorted: bool,
    pub skip_hidden: bool,
    pub max_depth: usize,
    pub max_file_cnt: usize,
    pub dir_include: Option<Vec<String>>,
    pub dir_exclude: Option<Vec<String>>,
    pub file_include: Option<Vec<String>>,
    pub file_exclude: Option<Vec<String>>,
    pub case_sensitive: bool,
    pub return_type: ReturnType,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub dir_include: Vec<Pattern>,
    pub dir_exclude: Vec<Pattern>,
    pub file_include: Vec<Pattern>,
    pub file_exclude: Vec<Pattern>,
    pub options: Option<MatchOptions>,
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DirEntry {
    pub path: String,
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    pub st_ctime: f64,
    pub st_mtime: f64,
    pub st_atime: f64,
    pub st_size: u64,
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DirEntryExt {
    pub path: String,
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    pub st_ctime: f64,
    pub st_mtime: f64,
    pub st_atime: f64,
    pub st_mode: u32,
    pub st_ino: u64,
    pub st_dev: u64,
    pub st_nlink: u64,
    pub st_size: u64,
    pub st_blksize: u64,
    pub st_blocks: u64,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
}

#[derive(Debug, Clone)]
pub enum ScandirResult {
    DirEntry(DirEntry),
    DirEntryExt(DirEntryExt),
    Error((String, String)),
}

pub type ScandirResultsType = Vec<ScandirResult>;
pub type ErrorsType = Vec<(String, String)>;

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
        self.dirs.is_empty()
            && self.files.is_empty()
            && self.symlinks.is_empty()
            && self.other.is_empty()
            && self.errors.is_empty()
    }

    pub fn extend(&mut self, root_dir: &str, other: &Toc) {
        self.dirs.extend_from_slice(
            &other
                .dirs
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>(),
        );
        self.files.extend_from_slice(
            &other
                .files
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>(),
        );
        self.symlinks.extend_from_slice(
            &other
                .symlinks
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>(),
        );
        self.other.extend_from_slice(
            &other
                .other
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>(),
        );
        self.errors.extend_from_slice(
            &other
                .errors
                .iter()
                .map(|x| PathBuf::from(root_dir).join(x).to_str().unwrap().to_owned())
                .collect::<Vec<String>>(),
        );
    }
}

impl Default for Toc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct WalkEntry {
    pub path: String,
    pub toc: Toc,
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct WalkEntryExt {
    pub path: String,
    pub toc: Toc,
}

#[derive(Debug, Clone)]
pub enum WalkResult {
    Toc(Toc),
    WalkEntry(WalkEntry),
    WalkEntryExt(WalkEntryExt),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ReturnType {
    Fast,
    Base,
    Ext,
    Walk,
}
