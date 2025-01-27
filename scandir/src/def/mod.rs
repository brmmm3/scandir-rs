use glob_sl::{MatchOptions, Pattern};

pub type ErrorsType = Vec<(String, String)>; // Tuple with file path and error message

pub mod count;
pub use count::Statistics;
pub mod direntry;
pub mod options;
pub mod walk;
pub use direntry::{DirEntry, DirEntryExt};
pub use options::Options;
pub mod scandir;
pub use scandir::ScandirResult;
pub mod toc;
pub use toc::Toc;

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
    Base,
    Ext,
}
