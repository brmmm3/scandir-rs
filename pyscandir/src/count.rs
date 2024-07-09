use std::io::ErrorKind;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;

#[cfg(any(feature = "speedy", feature = "bincode"))]
use pyo3::types::PyBytes;

#[cfg(feature = "speedy")]
use speedy::Writable;

use crate::def::{ReturnType, Statistics};

#[pyclass]
#[derive(Debug)]
pub struct Count {
    instance: scandir::Count,
    busy: bool,
}

#[pymethods]
impl Count {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (root_path, skip_hidden=None, max_depth=None, max_file_cnt=None, dir_include=None, dir_exclude=None, file_include=None, file_exclude=None, case_sensitive=None, return_type=None))]
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
                    ErrorKind::InvalidInput => {
                        return Err(PyValueError::new_err(e.to_string()));
                    }
                    ErrorKind::NotFound => {
                        return Err(PyFileNotFoundError::new_err(e.to_string()));
                    }
                    _ => {
                        return Err(PyException::new_err(e.to_string()));
                    }
                },
            },
            busy: false,
        })
    }

    pub fn extended(&mut self, extended: bool) {
        self.instance.set_extended(extended);
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
        Ok(Py::new(py, Statistics::from(&results))
            .unwrap()
            .to_object(py))
    }

    pub fn has_results(&mut self) -> bool {
        self.instance.has_results()
    }

    pub fn results(&mut self, py: Python) -> PyObject {
        Py::new(py, Statistics::from(&self.instance.results()))
            .unwrap()
            .to_object(py)
    }

    pub fn has_errors(&mut self) -> bool {
        self.instance.has_errors()
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

    #[pyo3(signature = (duration=None))]
    fn as_dict(&mut self, duration: Option<bool>, py: Python) -> PyResult<PyObject> {
        Statistics::from(&self.instance.results()).as_dict(duration, py)
    }

    #[cfg(feature = "speedy")]
    fn to_speedy(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.instance.statistics.write_to_vec() {
            Ok(v) => Ok(PyBytes::new_bound_with(py, v.len(), |b| {
                b.copy_from_slice(&v);
                Ok(())
            })?
            .into()),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "bincode")]
    fn to_bincode(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.instance.statistics.to_vec() {
            Ok(v) => Ok(PyBytes::new_bound_with(py, v.len(), |b| {
                b.copy_from_slice(&v);
                Ok(())
            })?
            .into()),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "json")]
    fn to_json(&self) -> PyResult<String> {
        self.instance
            .statistics
            .to_json()
            .map_err(|e| PyException::new_err(e.to_string()))
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
            Some(ty) => Python::with_gil(|py| ty.eq(py.get_type_bound::<PyValueError>())),
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
            Py::new(py, Statistics::from(&self.instance.results()))
                .unwrap()
                .to_object(py),
        ))
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }
}
