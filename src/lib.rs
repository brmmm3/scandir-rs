#![feature(specialization)]

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

mod def;
use def::*;
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
    m.add("ITER_TYPE_TOC", ITER_TYPE_TOC)?;
    m.add("ITER_TYPE_WALK", ITER_TYPE_WALK)?;
    m.add("ITER_TYPE_WALKEXT", ITER_TYPE_WALKEXT)?;
    m.add_wrapped(wrap_pymodule!(count))?;
    m.add_wrapped(wrap_pymodule!(walk))?;
    m.add_wrapped(wrap_pymodule!(scandir))?;
    Ok(())
}
