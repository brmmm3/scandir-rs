use std::io::ErrorKind;
use std::thread;
use std::time::Duration;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyType};
use pyo3::Python;

use crate::def::{DirEntry, DirEntryExt, ReturnType};
use scandir::{self, ScandirResult};

fn result2py(result: &ScandirResult, py: Python) -> PyObject {
    match result {
        ScandirResult::DirEntry(e) => PyCell::new(py, DirEntry::new(e)).unwrap().to_object(py),
        ScandirResult::DirEntryExt(e) => {
            PyCell::new(py, DirEntryExt::new(e)).unwrap().to_object(py)
        }
        ScandirResult::Error((path, e)) => (path.into_py(py), e.to_object(py)).to_object(py),
    }
}

#[pyclass]
#[derive(Debug)]
pub struct Scandir {
    instance: scandir::Scandir,
    entries: Vec<ScandirResult>,
    errors: Vec<(String, String)>,
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
    ) -> PyResult<Self> {
        Ok(Scandir {
            instance: match scandir::Scandir::new(
                root_path,
                sorted.unwrap_or(false),
                skip_hidden.unwrap_or(false),
                max_depth.unwrap_or(0) as i32,
                max_file_cnt.unwrap_or(0) as i32,
                dir_include,
                dir_exclude,
                file_include,
                file_exclude,
                case_sensitive.unwrap_or(false),
                return_type.unwrap_or(ReturnType::Base).from_object(),
            ) {
                Ok(s) => s,
                Err(e) => match e.kind() {
                    ErrorKind::InvalidInput => return Err(PyValueError::new_err(e.to_string())),
                    ErrorKind::NotFound => return Err(PyFileNotFoundError::new_err(e.to_string())),
                    _ => return Err(PyException::new_err(e.to_string())),
                },
            },
            entries: Vec::new(),
            errors: Vec::new(),
        })
    }

    pub fn clear(&mut self) {
        self.instance.clear();
        self.entries.clear();
        self.errors.clear();
    }

    pub fn start(&mut self) -> PyResult<bool> {
        if !self.instance.start() {
            return Err(PyRuntimeError::new_err("Thread already running"));
        }
        Ok(true)
    }

    pub fn join(&mut self) -> PyResult<bool> {
        if !self.instance.join() {
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

    pub fn collect(&mut self, py: Python) -> (Vec<PyObject>, Vec<(String, String)>) {
        let (entries, errors) = self.instance.collect();
        let results = entries.iter().map(|e| result2py(e, py)).collect();
        (results, errors)
    }

    pub fn results(
        &mut self,
        return_all: Option<bool>,
        py: Python,
    ) -> (Vec<PyObject>, Vec<(String, String)>) {
        let (entries, errors) = self.instance.results(return_all.unwrap_or(false));
        let results = entries.iter().map(|e| result2py(e, py)).collect();
        (results, errors)
    }

    pub fn entries(&mut self, return_all: bool, py: Python) -> Vec<PyObject> {
        self.instance
            .entries(return_all)
            .iter()
            .map(|e| result2py(e, py))
            .collect()
    }

    pub fn errors(&mut self, return_all: bool) -> Vec<(String, String)> {
        self.instance.errors(return_all)
    }

    pub fn duration(&mut self) -> f64 {
        self.instance.duration()
    }

    pub fn finished(&mut self) -> bool {
        self.instance.finished()
    }

    pub fn has_entries(&mut self) -> bool {
        self.instance.has_entries()
    }

    pub fn has_errors(&mut self) -> bool {
        self.instance.has_errors()
    }

    pub fn busy(&self) -> bool {
        self.instance.busy()
    }

    pub fn as_dict(&mut self, py: Python) -> PyObject {
        let pyresults = PyDict::new(py);
        for result in self.instance.entries(true) {
            let _ = match result {
                ScandirResult::DirEntry(e) => pyresults.set_item(
                    e.path.clone().into_py(py),
                    PyCell::new(py, DirEntry::new(&e)).unwrap().to_object(py),
                ),
                ScandirResult::DirEntryExt(e) => pyresults.set_item(
                    e.path.clone().into_py(py),
                    PyCell::new(py, DirEntryExt::new(&e)).unwrap().to_object(py),
                ),
                ScandirResult::Error((path, e)) => {
                    pyresults.set_item(path.into_py(py), e.to_object(py))
                }
            };
        }
        pyresults.to_object(py)
    }

    fn __enter__(&mut self) -> PyResult<()> {
        if !self.instance.start() {
            return Err(PyRuntimeError::new_err("Thread already running"));
        }
        Ok(())
    }

    fn __exit__(
        &mut self,
        ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        if !self.instance.stop() {
            return Ok(false);
        }
        match ty {
            Some(ty) => {
                if ty
                    .eq(Python::acquire_gil().python().get_type::<PyValueError>())
                    .unwrap()
                {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }

    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        if slf.instance.busy() {
            return Err(PyRuntimeError::new_err("Busy"));
        }
        if !slf.instance.start() {
            return Err(PyRuntimeError::new_err("Failed to start"));
        }
        slf.entries.clear();
        slf.errors.clear();
        Ok(slf)
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        loop {
            if let Some(entry) = self.entries.pop() {
                match entry {
                    ScandirResult::DirEntry(e) => {
                        return Ok(Some(
                            PyCell::new(py, DirEntry::new(&e)).unwrap().to_object(py),
                        ))
                    }
                    ScandirResult::DirEntryExt(e) => {
                        return Ok(Some(
                            PyCell::new(py, DirEntryExt::new(&e)).unwrap().to_object(py),
                        ))
                    }
                    ScandirResult::Error(error) => return Ok(Some(error.to_object(py))),
                }
            }
            if let Some(error) = self.errors.pop() {
                return Ok(Some(error.to_object(py)));
            }
            let (entries, errors) = self.instance.results(false);
            if entries.is_empty() && errors.is_empty() {
                if !self.instance.busy() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            } else {
                self.entries.extend_from_slice(&entries);
                self.errors.extend_from_slice(&errors);
            }
        }
        Ok(None)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
