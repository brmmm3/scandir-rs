//! `scandir` is a directory iteration module like `walk`, but with more features and higher speed. Depending on the function call
//! it yields a list of paths, tuple of lists grouped by their entry type or ``DirEntry`` objects that include file type and stat information along
//! with the name.
//!
//! If you are just interested in directory statistics you can use the ``Count``.
//!
//! `scandir` contains following classes:
//! - `Count` for determining statistics of a directory.
//! - `Walk` for getting names of directory entries.
//! - `Scandir` for getting detailed stats of directory entries.

#![cfg_attr(windows, feature(windows_by_handle))]

extern crate glob_sl;
#[macro_use]
extern crate serde_derive;

pub mod def;
pub use def::*;
pub mod common;
pub mod count;
pub use count::*;
pub mod walk;
pub use walk::*;
pub mod scandir;
pub use scandir::*;
