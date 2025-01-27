#[cfg(any(feature = "speedy", feature = "bincode", feature = "json"))]
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
#[cfg(any(feature = "speedy", feature = "bincode"))]
use pyo3::types::PyBytes;
use pyo3::types::PyDict;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Toc(scandir::Toc);

impl Toc {
    pub fn from(entry: &scandir::Toc) -> Self {
        Toc(entry.clone())
    }
}

#[pymethods]
impl Toc {
    #[getter]
    fn dirs(&self) -> Vec<String> {
        self.0.dirs()
    }

    #[getter]
    fn files(&self) -> Vec<String> {
        self.0.files()
    }

    #[getter]
    fn symlinks(&self) -> Vec<String> {
        self.0.symlinks()
    }

    #[getter]
    fn other(&self) -> Vec<String> {
        self.0.other()
    }

    #[getter]
    fn errors(&self) -> Vec<String> {
        self.0.errors()
    }

    fn as_dict(&self, py: Python) -> PyResult<PyObject> {
        let pydict = PyDict::new(py);
        pydict.set_item("dirs", self.0.dirs.clone())?;
        pydict.set_item("files", self.0.files.clone())?;
        pydict.set_item("symlinks", self.0.symlinks.clone())?;
        pydict.set_item("other", self.0.other.clone())?;
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
