use std::fmt::Debug;
use std::io::ErrorKind;
use std::thread;
use std::time::Duration;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};
use pyo3::Python;

use crate::def::{ReturnType, Toc};

#[pyclass]
#[derive(Debug)]
pub struct Walk {
    instance: scandir::Walk,
    return_type: ReturnType,
    // For iterator
    entries: Vec<(String, scandir::Toc)>,
    idx: usize,
}

#[pymethods]
impl Walk {
    #[new]
    fn new(
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
        let return_type = return_type.unwrap_or(ReturnType::Base);
        Ok(Walk {
            instance: match scandir::Walk::new(root_path) {
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
                    .return_type(return_type.from_object()),
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => return Err(PyFileNotFoundError::new_err(e.to_string())),
                    _ => return Err(PyException::new_err(e.to_string())),
                },
            },
            return_type,
            entries: Vec::new(),
            idx: std::usize::MAX,
        })
    }

    pub fn clear(&mut self) {
        self.instance.clear();
        self.entries.clear();
        self.idx = std::usize::MAX;
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

    pub fn collect(&mut self, store: Option<bool>, py: Python) -> PyResult<Toc> {
        Ok(Toc::new(Some(py.allow_threads(|| {
            self.instance.collect(store.unwrap_or(true))
        })?)))
    }

    pub fn has_results(&mut self, only_new: Option<bool>) -> bool {
        self.instance.has_results(only_new.unwrap_or(false))
    }

    pub fn results_cnt(&mut self) -> usize {
        self.instance.results_cnt()
    }

    pub fn results(
        &mut self,
        return_all: Option<bool>,
        store: Option<bool>,
        py: Python,
    ) -> Vec<(String, PyObject)> {
        let mut results = Vec::new();
        for result in self
            .instance
            .results(return_all.unwrap_or(false), store.unwrap_or(true))
        {
            results.push((
                result.0,
                PyCell::new(py, Toc::new(Some(result.1)))
                    .unwrap()
                    .to_object(py),
            ));
        }
        results
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
        if slf.idx < std::usize::MAX {
            return Err(PyRuntimeError::new_err("Busy"));
        }
        slf.instance.start()?;
        slf.entries.clear();
        slf.idx = 0;
        Ok(slf)
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        loop {
            if let Some((root_dir, toc)) = self.entries.get(self.idx) {
                self.idx += 1;
                if self.return_type == ReturnType::Base {
                    return Ok(Some(
                        (root_dir, toc.dirs.clone(), toc.files.clone()).to_object(py),
                    ));
                } else {
                    return Ok(Some(
                        (
                            root_dir,
                            toc.dirs.clone(),
                            toc.files.clone(),
                            toc.symlinks.clone(),
                            toc.other.clone(),
                            toc.errors.clone(),
                        )
                            .to_object(py),
                    ));
                }
            } else {
                self.entries.clear();
                self.entries
                    .extend_from_slice(&self.instance.results(false, true)[..]);
                if self.entries.is_empty() {
                    if !self.instance.busy() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                self.idx = 0;
            }
        }
        self.idx = std::usize::MAX;
        Ok(None)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
