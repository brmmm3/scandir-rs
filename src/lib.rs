#![feature(specialization)]

extern crate glob;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

mod def;
use def::*;
mod common;
pub mod count;
use count::*;
pub mod walk;
use walk::*;
pub mod scandir;
use scandir::*;
mod test;

#[pymodule(scandir_rs)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("RETURN_TYPE_FAST", RETURN_TYPE_FAST)?;
    m.add("RETURN_TYPE_BASE", RETURN_TYPE_BASE)?;
    m.add("RETURN_TYPE_WALK", RETURN_TYPE_WALK)?;
    m.add("RETURN_TYPE_EXT", RETURN_TYPE_EXT)?;
    m.add("RETURN_TYPE_FULL", RETURN_TYPE_FULL)?;
    m.add_wrapped(wrap_pymodule!(count))?;
    m.add_wrapped(wrap_pymodule!(walk))?;
    m.add_wrapped(wrap_pymodule!(scandir))?;
    Ok(())
}
