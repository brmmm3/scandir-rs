#![feature(specialization)]

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

mod def;
mod count;
use count::*;
mod walk;
use walk::*;
mod scandir;
use scandir::*;
mod test;

#[pymodule(scandir_rs)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_wrapped(wrap_pymodule!(count))?;
    m.add_wrapped(wrap_pymodule!(walk))?;
    m.add_wrapped(wrap_pymodule!(scandir))?;
    Ok(())
}
