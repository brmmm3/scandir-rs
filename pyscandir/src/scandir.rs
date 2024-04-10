use std::io::ErrorKind;
use std::thread;
use std::time::Duration;

use pyo3::exceptions::{ PyException, PyFileNotFoundError, PyRuntimeError, PyValueError };
use pyo3::prelude::*;
use pyo3::types::{ PyDict, PyType, PyBytes };
use scandir::def::scandir::ScandirResults;

use crate::def::{ DirEntry, DirEntryExt, ReturnType, Statistics };
use scandir::{ ErrorsType, ScandirResult };

fn result2py(result: &ScandirResult, py: Python) -> Option<PyObject> {
    match result {
        ScandirResult::DirEntry(e) => {
            Some(Py::new(py, DirEntry::from(e)).unwrap().to_object(py))
        }
        ScandirResult::DirEntryExt(e) => {
            Some(Py::new(py, DirEntryExt::from(e)).unwrap().to_object(py))
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
    #[new]
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
        return_type: Option<ReturnType>,
        store: Option<bool>
    ) -> PyResult<Self> {
        let return_type = return_type.unwrap_or(ReturnType::Base).from_object();
        Ok(Scandir {
            instance: match scandir::Scandir::new(root_path, store) {
                Ok(s) =>
                    s
                        .sorted(sorted.unwrap_or(false))
                        .skip_hidden(skip_hidden.unwrap_or(false))
                        .max_depth(max_depth.unwrap_or(0))
                        .max_file_cnt(max_file_cnt.unwrap_or(0))
                        .dir_include(dir_include)
                        .dir_exclude(dir_exclude)
                        .file_include(file_include)
                        .file_exclude(file_exclude)
                        .case_sensitive(case_sensitive.unwrap_or(false))
                        .return_type(return_type),
                Err(e) =>
                    match e.kind() {
                        ErrorKind::NotFound => {
                            return Err(PyFileNotFoundError::new_err(e.to_string()));
                        }
                        _ => {
                            return Err(PyException::new_err(e.to_string()));
                        }
                    }
            },
            entries: ScandirResults::new(),
        })
    }

    pub fn clear(&mut self) {
        self.instance.clear();
        self.entries.clear();
    }

    pub fn start(&mut self) -> PyResult<()> {
        self.instance.start().map_err(|e| PyException::new_err(e.to_string()))
    }

    pub fn join(&mut self, py: Python) -> PyResult<bool> {
        let result = py.allow_threads(|| self.instance.join());
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

    pub fn collect(&mut self, py: Python) -> PyResult<(Vec<PyObject>, ErrorsType)> {
        let entries = py.allow_threads(|| self.instance.collect())?;
        let results = entries.results
            .iter()
            .filter_map(|r| result2py(r, py))
            .collect();
        Ok((results, entries.errors))
    }

    pub fn has_results(&mut self, only_new: Option<bool>) -> bool {
        self.instance.has_results(only_new.unwrap_or(true))
    }

    pub fn results_cnt(&mut self, only_new: Option<bool>) -> usize {
        self.instance.results_cnt(only_new.unwrap_or(true))
    }

    pub fn results(&mut self, only_new: Option<bool>, py: Python) -> (Vec<PyObject>, ErrorsType) {
        let entries = self.instance.results(only_new.unwrap_or(true));
        let results = entries.results
            .iter()
            .filter_map(|e| result2py(e, py))
            .collect();
        (results, entries.errors)
    }

    pub fn has_entries(&mut self, only_new: Option<bool>) -> bool {
        self.instance.has_entries(only_new.unwrap_or(true))
    }

    pub fn entries_cnt(&mut self, only_new: Option<bool>) -> usize {
        self.instance.entries_cnt(only_new.unwrap_or(true))
    }

    pub fn entries(&mut self, only_new: Option<bool>, py: Python) -> Vec<PyObject> {
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

    pub fn errors(&mut self, only_new: Option<bool>) -> ErrorsType {
        self.instance.errors(only_new.unwrap_or(true))
    }

    pub fn as_dict(&mut self, only_new: Option<bool>, py: Python) -> PyObject {
        let pyresults = PyDict::new_bound(py);
        let entries = self.instance.results(only_new.unwrap_or(true));
        for entry in entries.results {
            let _ = match entry {
                ScandirResult::DirEntry(e) =>
                    pyresults.set_item(
                        e.path.clone().into_py(py),
                        Py::new(py, DirEntry::from(&e)).unwrap().to_object(py)
                    ),
                ScandirResult::DirEntryExt(e) =>
                    pyresults.set_item(
                        e.path.clone().into_py(py),
                        Py::new(py, DirEntryExt::from(&e)).unwrap().to_object(py)
                    ),
                ScandirResult::Error((path, e)) => {
                    pyresults.set_item(path.into_py(py), e.to_object(py))
                }
            };
        }
        for error in entries.errors {
            let _ = pyresults.set_item(error.0.into_py(py), error.1.to_object(py));
        }
        pyresults.to_object(py)
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self, py: Python) -> PyResult<Py<PyBytes>> {
        self.instance
            .to_speedy()
            .map(|v| {
                PyBytes::new_bound_with(py, v.len(), |b| {
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
                PyBytes::new_bound_with(py, v.len(), |b| {
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
        self.instance.to_json().map_err(|e| PyException::new_err(e.to_string()))
    }

    #[getter]
    pub fn statistics(&self) -> Statistics {
        Statistics(self.instance.statistics())
    }

    pub fn duration(&mut self) -> f64 {
        self.instance.duration()
    }

    pub fn finished(&mut self) -> bool {
        self.instance.finished()
    }

    pub fn busy(&self) -> bool {
        self.instance.busy()
    }

    fn __enter__(mut slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        slf.instance.start().map_err(|e| PyException::new_err(e.to_string()))?;
        Ok(slf)
    }

    fn __exit__(
        &mut self,
        ty: Option<Bound<PyType>>,
        _value: Option<Bound<PyAny>>,
        _traceback: Option<Bound<PyAny>>
    ) -> PyResult<bool> {
        if !self.instance.stop() {
            return Ok(false);
        }
        self.instance.join();
        match ty {
            Some(ty) => Python::with_gil(|py| ty.eq(py.get_type_bound::<PyValueError>())),
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

    fn __next__(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        loop {
            if let Some(entry) = self.entries.results.pop() {
                match entry {
                    ScandirResult::DirEntry(e) => {
                        return Ok(Some(Py::new(py, DirEntry::from(&e)).unwrap().to_object(py)));
                    }
                    ScandirResult::DirEntryExt(e) => {
                        return Ok(Some(Py::new(py, DirEntryExt::from(&e)).unwrap().to_object(py)));
                    }
                    ScandirResult::Error(error) => {
                        return Ok(Some(error.to_object(py)));
                    }
                }
            }
            if let Some(error) = self.entries.errors.pop() {
                return Ok(Some(error.to_object(py)));
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

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{self:?}"))
    }
}
