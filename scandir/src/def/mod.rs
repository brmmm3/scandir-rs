use std::path::PathBuf;

use glob_sl::{ MatchOptions, Pattern };

pub type ErrorsType = Vec<(String, String)>; // Tuple with file path and error message

pub mod count;
pub use count::Statistics;
pub mod walk;
pub mod direntry;
pub use direntry::{ DirEntry, DirEntryExt };
pub mod scandir;
pub use scandir::ScandirResult;
pub mod toc;
pub use toc::Toc;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub dir_include: Vec<Pattern>,
    pub dir_exclude: Vec<Pattern>,
    pub file_include: Vec<Pattern>,
    pub file_exclude: Vec<Pattern>,
    pub options: Option<MatchOptions>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ReturnType {
    Fast,
    Base,
    Ext,
    Walk,
}
