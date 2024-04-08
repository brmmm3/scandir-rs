use pyo3::prelude::*;

pub mod count;
pub use count::Statistics;
pub mod walk;
pub mod direntry;
pub use direntry::{ DirEntry, DirEntryExt };
pub mod scandir;
pub mod toc;
pub use toc::Toc;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub enum ReturnType {
    Base,
    Ext,
}

impl ReturnType {
    pub fn from_object(&self) -> ::scandir::ReturnType {
        match &self {
            ReturnType::Base => ::scandir::ReturnType::Base,
            ReturnType::Ext => ::scandir::ReturnType::Ext,
        }
    }
}
