#[cfg(any(feature = "speedy", feature = "bincode", feature = "json"))]
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
#[cfg(any(feature = "speedy", feature = "bincode"))]
use pyo3::types::PyBytes;
use pyo3::types::PyDict;

use crate::direntry::DirEntryExt;
use std::sync::Arc;

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct ScandirResult(scandir::ScandirResult);

#[pymethods]
impl ScandirResult {
    #[getter]
    fn path(&self) -> &str {
        self.0.path()
    }

    #[getter]
    fn error(&self) -> Option<&(String, String)> {
        self.0.error()
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

/// Convert a slice of ScandirResult to Python objects
fn results_to_py(results: &[scandir::ScandirResult], py: Python) -> Vec<Py<PyAny>> {
    results
        .iter()
        .filter_map(|r| result2py_inner(r, py))
        .collect()
}

fn result2py_inner(result: &scandir::ScandirResult, py: Python) -> Option<Py<PyAny>> {
    match result {
        scandir::ScandirResult::DirEntry(e) => Some(
            Py::new(py, crate::direntry::DirEntry::from(e))
                .unwrap()
                .into_any(),
        ),
        scandir::ScandirResult::DirEntryExt(e) => {
            Some(Py::new(py, DirEntryExt::from(e)).unwrap().into_any())
        }
        scandir::ScandirResult::Error(_) => None,
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct ScandirResults {
    inner: Arc<scandir::ScandirResults>,
}

impl ScandirResults {
    pub fn from_inner(inner: Arc<scandir::ScandirResults>) -> Self {
        ScandirResults { inner }
    }

    pub fn inner(&self) -> &scandir::ScandirResults {
        &self.inner
    }
}

#[pymethods]
impl ScandirResults {
    #[getter]
    fn results(&self, py: Python) -> Vec<Py<PyAny>> {
        results_to_py(&self.inner.results, py)
    }

    #[getter]
    fn dirs(&self, py: Python) -> Vec<Py<PyAny>> {
        self.inner
            .dirs()
            .filter_map(|r| result2py_inner(r, py))
            .collect()
    }

    #[getter]
    fn files(&self, py: Python) -> Vec<Py<PyAny>> {
        self.inner
            .files()
            .filter_map(|r| result2py_inner(r, py))
            .collect()
    }

    #[getter]
    fn symlinks(&self, py: Python) -> Vec<Py<PyAny>> {
        self.inner
            .symlinks()
            .filter_map(|r| result2py_inner(r, py))
            .collect()
    }

    #[getter]
    fn other(&self, py: Python) -> Vec<Py<PyAny>> {
        self.inner
            .other()
            .filter_map(|r| result2py_inner(r, py))
            .collect()
    }

    #[getter]
    fn errors(&self) -> Vec<(String, String)> {
        self.inner.errors.clone()
    }

    fn as_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let pydict = PyDict::new(py);
        for entry in &self.inner.results {
            let _ = match entry {
                scandir::ScandirResult::DirEntry(e) => {
                    let path = e.path.clone();
                    pydict.set_item(
                        path,
                        Py::new(py, crate::direntry::DirEntry::from_owned(e.clone()))
                            .unwrap()
                            .into_any(),
                    )
                }
                scandir::ScandirResult::DirEntryExt(e) => {
                    let path = e.path.clone();
                    pydict.set_item(
                        path,
                        Py::new(py, DirEntryExt::from_owned(e.clone()))
                            .unwrap()
                            .into_any(),
                    )
                }
                scandir::ScandirResult::Error((path, e)) => pydict.set_item(path, e),
            };
        }
        for error in &self.inner.errors {
            let _ = pydict.set_item(error.0.clone(), error.1.clone());
        }
        Ok(pydict.into_any().unbind())
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __bool__(&self) -> bool {
        !self.inner.is_empty()
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }
}
