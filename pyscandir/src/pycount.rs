use std::io::ErrorKind;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};
use pyo3::Python;

use crate::def::{ReturnType, Statistics};

#[pyclass]
#[derive(Debug)]
pub struct Count {
    instance: scandir::Count,
    busy: bool,
}

#[pymethods]
impl Count {
    #[new]
    fn new(
        root_path: &str,
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
        Ok(Count {
            instance: match scandir::Count::new(root_path) {
                Ok(c) => c
                    .skip_hidden(skip_hidden.unwrap_or(false))
                    .max_depth(max_depth.unwrap_or(0))
                    .max_file_cnt(max_file_cnt.unwrap_or(0))
                    .dir_include(dir_include)
                    .dir_exclude(dir_exclude)
                    .file_include(file_include)
                    .file_exclude(file_exclude)
                    .case_sensitive(case_sensitive.unwrap_or(false))
                    .extended(return_type.unwrap_or(ReturnType::Base) == ReturnType::Ext),
                Err(e) => match e.kind() {
                    ErrorKind::InvalidInput => return Err(PyValueError::new_err(e.to_string())),
                    ErrorKind::NotFound => return Err(PyFileNotFoundError::new_err(e.to_string())),
                    _ => return Err(PyException::new_err(e.to_string())),
                },
            },
            busy: false,
        })
    }

    pub fn clear(&mut self) {
        self.instance.clear();
    }

    pub fn start(&mut self) -> PyResult<()> {
        self.instance
            .start()
            .map_err(|e| PyException::new_err(e.to_string()))
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

    pub fn collect(&mut self, py: Python) -> PyResult<PyObject> {
        let results = py.allow_threads(|| self.instance.collect())?;
        Ok(PyCell::new(py, Statistics::new(Some(results)))
            .unwrap()
            .to_object(py))
    }

    pub fn has_results(&mut self) -> bool {
        self.instance.has_results()
    }

    pub fn results(&mut self, py: Python) -> PyObject {
        PyCell::new(py, Statistics::new(Some(self.instance.results())))
            .unwrap()
            .to_object(py)
    }

    pub fn has_errors(&mut self) -> bool {
        self.instance.has_errors()
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

    fn as_dict(&mut self, duration: Option<bool>, py: Python) -> PyResult<PyObject> {
        Statistics::new(Some(self.instance.results())).as_dict(duration, py)
    }

    fn __enter__(mut slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        slf.instance
            .start()
            .map_err(|e| PyException::new_err(e.to_string()))?;
        Ok(slf)
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
        self.instance.join();
        match ty {
            Some(ty) => Python::with_gil(|py| ty.eq(py.get_type::<PyValueError>())),
            None => Ok(false),
        }
    }

    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        if slf.busy {
            return Err(PyRuntimeError::new_err("Busy"));
        }
        slf.instance.start()?;
        slf.busy = true;
        Ok(slf)
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        if !self.busy {
            return Ok(None);
        }
        if !self.instance.busy() {
            self.busy = false;
        }
        Ok(Some(
            PyCell::new(py, Statistics::new(Some(self.instance.results())))
                .unwrap()
                .to_object(py),
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
