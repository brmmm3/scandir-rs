use pyo3::prelude::*;
use pyo3::types::PyDict;
#[cfg(any(feature = "speedy", feature = "bincode"))]
use pyo3::types::PyBytes;
#[cfg(any(feature = "speedy", feature = "bincode", feature = "json"))]
use pyo3::exceptions::PyException;

#[cfg(feature = "speedy")]
use speedy::Writable;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Statistics(pub scandir::Statistics);

impl Statistics {
    pub fn from(entry: &scandir::Statistics) -> Self {
        Statistics(entry.clone())
    }
}

#[pymethods]
impl Statistics {
    #[getter]
    fn dirs(&self) -> i32 {
        self.0.dirs
    }

    #[getter]
    fn files(&self) -> i32 {
        self.0.files
    }

    #[getter]
    fn slinks(&self) -> i32 {
        self.0.slinks
    }

    #[getter]
    fn hlinks(&self) -> i32 {
        self.0.hlinks
    }

    #[getter]
    fn devices(&self) -> i32 {
        self.0.devices
    }

    #[getter]
    fn pipes(&self) -> i32 {
        self.0.pipes
    }

    #[getter]
    fn size(&self) -> u64 {
        self.0.size
    }

    #[getter]
    fn usage(&self) -> u64 {
        self.0.usage
    }

    #[getter]
    fn errors(&self) -> Vec<String> {
        self.0.errors.clone()
    }

    #[getter]
    fn duration(&self) -> f64 {
        self.0.duration
    }

    pub fn as_dict(&self, duration: Option<bool>, py: Python) -> PyResult<PyObject> {
        let pyresult = PyDict::new_bound(py);
        if self.0.dirs > 0 {
            pyresult.set_item("dirs", self.0.dirs).unwrap();
        }
        if self.0.files > 0 {
            pyresult.set_item("files", self.0.files).unwrap();
        }
        if self.0.slinks > 0 {
            pyresult.set_item("slinks", self.0.slinks).unwrap();
        }
        if self.0.hlinks > 0 {
            pyresult.set_item("hlinks", self.0.hlinks).unwrap();
        }
        if self.0.devices > 0 {
            pyresult.set_item("devices", self.0.devices).unwrap();
        }
        if self.0.pipes > 0 {
            pyresult.set_item("pipes", self.0.pipes).unwrap();
        }
        if self.0.size > 0 {
            pyresult.set_item("size", self.0.size).unwrap();
        }
        if self.0.usage > 0 {
            pyresult.set_item("usage", self.0.usage).unwrap();
        }
        if !self.0.errors.is_empty() {
            pyresult.set_item("errors", self.0.errors.to_vec()).unwrap();
        }
        if duration.unwrap_or(false) {
            pyresult.set_item("duration", self.0.duration).unwrap();
        }
        Ok(pyresult.to_object(py))
    }

    #[cfg(feature = "speedy")]
    fn to_speedy(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.0.write_to_vec() {
            Ok(v) => {
                Ok(
                    PyBytes::new_bound_with(py, v.len(), |b| {
                        b.copy_from_slice(&v);
                        Ok(())
                    })?.into()
                )
            }
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "bincode")]
    fn to_bincode(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.0.to_vec() {
            Ok(v) => {
                Ok(
                    PyBytes::new_bound_with(py, v.len(), |b| {
                        b.copy_from_slice(&v);
                        Ok(())
                    })?.into()
                )
            }
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "json")]
    fn to_json(&self) -> PyResult<String> {
        self.0.to_json().map_err(|e| PyException::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }
}
