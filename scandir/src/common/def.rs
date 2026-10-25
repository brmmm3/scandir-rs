use std::fs::Metadata;

use glob_sl::{MatchOptions, Pattern};

pub type ErrorsType = Vec<(String, String)>; // Tuple with file path and error message

pub type DirEntryType = jwalk_meta::DirEntry<((), Option<Result<Metadata, std::io::Error>>)>;

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub dir_include: Vec<Pattern>,
    pub dir_exclude: Vec<Pattern>,
    pub file_include: Vec<Pattern>,
    pub file_exclude: Vec<Pattern>,
    pub options: Option<MatchOptions>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReturnType {
    Base,
    Ext,
}
