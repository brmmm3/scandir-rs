#![cfg_attr(windows, feature(windows_by_handle))]

extern crate glob;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

pub mod cst;
pub use cst::*;
pub mod def;
pub use def::*;
pub mod common;
pub mod count;
pub use count::*;
pub mod walk;
pub use walk::*;
pub mod scandir;
pub use scandir::*;

/// scandir_rs is a directory iteration module like os.walk(), but with more features and higher speed. Depending on the function call
/// it yields a list of paths, tuple of lists grouped by their entry type or DirEntry objects that include file type and stat information along
/// with the name. Using scandir_rs is about 2-17 times faster than os.walk() (depending on the platform, file system and file tree structure)
/// by parallelizing the iteration in background.
#[pymodule]
#[pyo3(name = "scandir_rs")]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("RETURN_TYPE_FAST", RETURN_TYPE_FAST)?;
    m.add("RETURN_TYPE_BASE", RETURN_TYPE_BASE)?;
    m.add("RETURN_TYPE_WALK", RETURN_TYPE_WALK)?;
    m.add("RETURN_TYPE_EXT", RETURN_TYPE_EXT)?;
    m.add("RETURN_TYPE_FULL", RETURN_TYPE_FULL)?;
    m.add_wrapped(wrap_pymodule!(count::init))?;
    m.add_wrapped(wrap_pymodule!(walk::init))?;
    m.add_wrapped(wrap_pymodule!(scandir::init))?;
    Ok(())
}
