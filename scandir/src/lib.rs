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

extern crate glob_sl;
#[cfg_attr(any(feature = "bincode", feature = "json"), macro_use)]
#[cfg(any(feature = "bincode", feature = "json"))]
extern crate serde_derive;

mod common;
pub use common::{
    DirEntryType, ErrorsType, Filter, ReturnType, check_and_expand_path, create_filter,
    filter_children, filter_direntry, get_root_path_len, start,
};
mod direntry;
pub use direntry::{DirEntry, DirEntryExt};
mod options;
pub use options::Options;
pub mod toc;
pub use toc::Toc;
pub mod count;
pub use count::{Count, Statistics};
pub mod scandir;
pub use scandir::{Scandir, ScandirResult, ScandirResults};
pub mod walk;
pub use walk::{Walk, WalkEntry, WalkEntryExt, WalkResult};
