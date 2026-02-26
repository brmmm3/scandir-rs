#[cfg(any(feature = "speedy", feature = "bincode", feature = "json"))]
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
#[cfg(any(feature = "speedy", feature = "bincode"))]
use pyo3::types::PyBytes;

use super::DirEntryExt;

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct ScandirResult(scandir::ScandirResult);

#[pymethods]
impl ScandirResult {
    #[getter]
    fn path(&self) -> String {
        self.0.path().clone()
    }

    #[getter]
    fn error(&self) -> Option<(String, String)> {
        self.0.error().cloned()
    }

    #[getter]
    fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    #[getter]
    fn is_file(&self) -> bool {
        self.0.is_file()
    }

    #[getter]
    fn is_symlink(&self) -> bool {
        self.0.is_symlink()
    }

    #[getter]
    fn ctime(&self) -> f64 {
        self.0.ctime()
    }

    #[getter]
    fn mtime(&self) -> f64 {
        self.0.mtime()
    }

    #[getter]
    fn atime(&self) -> f64 {
        self.0.atime()
    }

    #[getter]
    fn size(&self) -> u64 {
        self.0.size()
    }

    #[getter]
    fn ext(&self) -> Option<DirEntryExt> {
        match &self.0 {
            scandir::ScandirResult::DirEntryExt(e) => Some(DirEntryExt::from(e)),
            _ => None,
        }
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
