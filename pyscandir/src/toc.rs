#[cfg(any(feature = "speedy", feature = "bincode", feature = "json"))]
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
#[cfg(any(feature = "speedy", feature = "bincode"))]
use pyo3::types::PyBytes;
use pyo3::types::PyDict;

use crate::common::paths_to_os_strings;

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct Toc(scandir::Toc);

impl Toc {
    pub fn from(entry: &scandir::Toc) -> Self {
        Toc(entry.clone())
    }

    pub fn from_owned(entry: scandir::Toc) -> Self {
        Toc(entry)
    }
}

#[pymethods]
impl Toc {
    #[getter]
    fn dirs(&self) -> Vec<std::ffi::OsString> {
        paths_to_os_strings(self.0.dirs())
    }

    #[getter]
    fn files(&self) -> Vec<std::ffi::OsString> {
        paths_to_os_strings(self.0.files())
    }

    #[getter]
    fn symlinks(&self) -> Vec<std::ffi::OsString> {
        paths_to_os_strings(self.0.symlinks())
    }

    #[getter]
    fn other(&self) -> Vec<std::ffi::OsString> {
        paths_to_os_strings(self.0.other())
    }

    #[getter]
    fn errors(&self) -> Vec<String> {
        self.0.errors()
    }

    fn as_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let pydict = PyDict::new(py);
        pydict.set_item("dirs", self.dirs())?;
        pydict.set_item("files", self.files())?;
        pydict.set_item("symlinks", self.symlinks())?;
        pydict.set_item("other", self.other())?;
        pydict.set_item("errors", self.0.errors.clone())?;
        Ok(pydict.into_any().unbind())
    }

    #[cfg(feature = "speedy")]
    fn to_speedy(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.0.to_speedy() {
            Ok(v) => Ok(PyBytes::new_with(py, v.len(), |b| {
                b.copy_from_slice(&v);
                Ok(())
            })?
            .into()),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "bincode")]
    fn to_bincode(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.0.to_bincode() {
            Ok(v) => Ok(PyBytes::new_with(py, v.len(), |b| {
                b.copy_from_slice(&v);
                Ok(())
            })?
            .into()),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "json")]
    fn to_json(&self) -> PyResult<String> {
        self.0
            .to_json()
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }
}
