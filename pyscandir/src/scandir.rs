use std::io::ErrorKind;
use std::thread;
use std::time::Duration;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::types::{PyBytes, PyDict, PyType};
use pyo3::{IntoPyObjectExt, prelude::*};
use scandir::def::scandir::ScandirResults;

use crate::def::{DirEntry, DirEntryExt, ReturnType, Statistics};
use scandir::{ErrorsType, ScandirResult};

fn result2py(result: &ScandirResult, py: Python) -> Option<Py<PyAny>> {
    match result {
        ScandirResult::DirEntry(e) => Some(Py::new(py, DirEntry::from(e)).unwrap().into_any()),
        ScandirResult::DirEntryExt(e) => {
            Some(Py::new(py, DirEntryExt::from(e)).unwrap().into_any())
        }
        ScandirResult::Error((_path, _e)) => None,
    }
}

#[pyclass]
#[derive(Debug)]
pub struct Scandir {
    instance: scandir::Scandir,
    entries: ScandirResults,
}

#[pymethods]
impl Scandir {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (root_path, sorted=None, skip_hidden=None, max_depth=None, max_file_cnt=None, dir_include=None, dir_exclude=None, file_include=None, file_exclude=None, case_sensitive=None, follow_links=None, return_type=None, store=None))]
    pub fn new(
        root_path: &str,
        sorted: Option<bool>,
        skip_hidden: Option<bool>,
        max_depth: Option<usize>,
        max_file_cnt: Option<usize>,
        dir_include: Option<Vec<String>>,
        dir_exclude: Option<Vec<String>>,
        file_include: Option<Vec<String>>,
        file_exclude: Option<Vec<String>>,
        case_sensitive: Option<bool>,
        follow_links: Option<bool>,
        return_type: Option<ReturnType>,
        store: Option<bool>,
    ) -> PyResult<Self> {
        let return_type = return_type.unwrap_or(ReturnType::Base).from_object();
        Ok(Scandir {
            instance: match scandir::Scandir::new(root_path, store) {
                Ok(s) => s
                    .sorted(sorted.unwrap_or(false))
                    .skip_hidden(skip_hidden.unwrap_or(false))
                    .max_depth(max_depth.unwrap_or(0))
                    .max_file_cnt(max_file_cnt.unwrap_or(0))
                    .dir_include(dir_include)
                    .dir_exclude(dir_exclude)
                    .file_include(file_include)
                    .file_exclude(file_exclude)
                    .case_sensitive(case_sensitive.unwrap_or(false))
                    .follow_links(follow_links.unwrap_or(false))
                    .return_type(return_type),
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => {
                        return Err(PyFileNotFoundError::new_err(e.to_string()));
                    }
                    _ => {
                        return Err(PyException::new_err(e.to_string()));
                    }
                },
            },
            entries: ScandirResults::new(),
        })
    }

    pub fn extended(&mut self, extended: bool) {
        self.instance.set_extended(extended);
    }

    pub fn clear(&mut self) {
        self.instance.clear();
        self.entries.clear();
    }

    pub fn start(&mut self) -> PyResult<()> {
        self.instance
            .start()
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    pub fn join(&mut self, py: Python) -> PyResult<bool> {
        let result = py.detach(|| self.instance.join());
        if !result {
            return Err(PyRuntimeError::new_err("Thread not running"));
        }
        Ok(true)
    }

    pub fn stop(&mut self) -> PyResult<bool> {
        if !self.instance.stop() {
            return Err(PyRuntimeError::new_err("Thread not running"));
        }
        Ok(true)
    }

    pub fn collect(&mut self, py: Python) -> PyResult<(Vec<Py<PyAny>>, ErrorsType)> {
        let entries = py.detach(|| self.instance.collect())?;
        let results = entries
            .results
            .iter()
            .filter_map(|r| result2py(r, py))
            .collect();
        Ok((results, entries.errors))
    }

    #[pyo3(signature = (only_new=None))]
    pub fn has_results(&mut self, only_new: Option<bool>) -> bool {
        self.instance.has_results(only_new.unwrap_or(true))
    }

    #[pyo3(signature = (only_new=None))]
    pub fn results_cnt(&mut self, only_new: Option<bool>) -> usize {
        self.instance.results_cnt(only_new.unwrap_or(true))
    }

    #[pyo3(signature = (only_new=None))]
    pub fn results(&mut self, only_new: Option<bool>, py: Python) -> (Vec<Py<PyAny>>, ErrorsType) {
        let entries = self.instance.results(only_new.unwrap_or(true));
        let results = entries
            .results
            .iter()
            .filter_map(|e| result2py(e, py))
            .collect();
        (results, entries.errors)
    }

    #[pyo3(signature = (only_new=None))]
    pub fn has_entries(&mut self, only_new: Option<bool>) -> bool {
        self.instance.has_entries(only_new.unwrap_or(true))
    }

    #[pyo3(signature = (only_new=None))]
    pub fn entries_cnt(&mut self, only_new: Option<bool>) -> usize {
        self.instance.entries_cnt(only_new.unwrap_or(true))
    }

    #[pyo3(signature = (only_new=None))]
    pub fn entries(&mut self, only_new: Option<bool>, py: Python) -> Vec<Py<PyAny>> {
        self.instance
            .entries(only_new.unwrap_or(true))
            .iter()
            .filter_map(|e| result2py(e, py))
            .collect()
    }

    pub fn has_errors(&mut self) -> bool {
        self.instance.has_errors()
    }

    pub fn errors_cnt(&mut self) -> usize {
        self.instance.errors_cnt()
    }

    #[pyo3(signature = (only_new=None))]
    pub fn errors(&mut self, only_new: Option<bool>) -> ErrorsType {
        self.instance.errors(only_new.unwrap_or(true))
    }

    #[pyo3(signature = (only_new=None))]
    pub fn as_dict(&mut self, only_new: Option<bool>, py: Python) -> PyResult<Py<PyAny>> {
        let pyresults = PyDict::new(py);
        let entries = self.instance.results(only_new.unwrap_or(true));
        for entry in entries.results {
            let _ = match entry {
                ScandirResult::DirEntry(e) => pyresults.set_item(
                    e.path.clone().into_py_any(py)?,
                    Py::new(py, DirEntry::from(&e)).unwrap().into_any(),
                ),
                ScandirResult::DirEntryExt(e) => pyresults.set_item(
                    e.path.clone().into_py_any(py)?,
                    Py::new(py, DirEntryExt::from(&e)).unwrap().into_any(),
                ),
                ScandirResult::Error((path, e)) => pyresults.set_item(path.into_py_any(py)?, e),
            };
        }
        for error in entries.errors {
            let _ = pyresults.set_item(error.0.into_py_any(py)?, error.1.into_py_any(py)?);
        }
        Ok(pyresults.into_any().unbind())
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self, py: Python) -> PyResult<Py<PyBytes>> {
        self.instance
            .to_speedy()
            .map(|v| {
                PyBytes::new_with(py, v.len(), |b| {
                    b.copy_from_slice(&v);
                    Ok(())
                })
                .unwrap()
                .into()
            })
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    #[cfg(feature = "bincode")]
    pub fn to_bincode(&self, py: Python) -> PyResult<Py<PyBytes>> {
        self.instance
            .to_bincode()
            .map(|v| {
                PyBytes::new_with(py, v.len(), |b| {
                    b.copy_from_slice(&v);
                    Ok(())
                })
                .unwrap()
                .into()
            })
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> PyResult<String> {
        self.instance
            .to_json()
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    #[getter]
    pub fn statistics(&self) -> Statistics {
        Statistics(self.instance.statistics())
    }

    #[getter]
    pub fn duration(&mut self) -> f64 {
        self.instance.duration()
    }

    #[getter]
    pub fn finished(&mut self) -> bool {
        self.instance.finished()
    }

    #[getter]
    pub fn busy(&self) -> bool {
        self.instance.busy()
    }

    fn __enter__(mut slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        slf.instance
            .start()
            .map_err(|e| PyException::new_err(e.to_string()))?;
        Ok(slf)
    }

    #[pyo3(signature = (ty=None, _value=None, _traceback=None))]
    fn __exit__(
        &mut self,
        ty: Option<&Bound<PyType>>,
        _value: Option<&Bound<PyAny>>,
        _traceback: Option<&Bound<PyAny>>,
    ) -> PyResult<bool> {
        if !self.instance.stop() {
            return Ok(false);
        }
        self.instance.join();
        match ty {
            Some(ty) => Python::attach(|py| ty.eq(py.get_type::<PyValueError>())),
            None => Ok(false),
        }
    }

    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        if slf.instance.busy() {
            return Err(PyRuntimeError::new_err("Busy"));
        }
        slf.instance.start()?;
        slf.entries.clear();
        Ok(slf)
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<Py<PyAny>>> {
        loop {
            if let Some(entry) = self.entries.results.pop() {
                match entry {
                    ScandirResult::DirEntry(e) => {
                        return Ok(Some(Py::new(py, DirEntry::from(&e)).unwrap().into_any()));
                    }
                    ScandirResult::DirEntryExt(e) => {
                        return Ok(Some(Py::new(py, DirEntryExt::from(&e)).unwrap().into_any()));
                    }
                    ScandirResult::Error(error) => {
                        return Ok(Some(error.into_py_any(py)?));
                    }
                }
            }
            if let Some(error) = self.entries.errors.pop() {
                return Ok(Some(error.into_py_any(py)?));
            }
            let entries = self.instance.results(true);
            if entries.is_empty() {
                if !self.instance.busy() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            } else {
                self.entries.extend(&entries);
            }
        }
        Ok(None)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }
}
