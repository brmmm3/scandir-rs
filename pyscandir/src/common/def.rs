use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::time::{Duration, Instant};

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Debug, Clone, PartialEq)]
pub enum ReturnType {
    Base,
    Ext,
}

impl ReturnType {
    #[allow(clippy::wrong_self_convention)]
    pub fn from_object(&self) -> ::scandir::ReturnType {
        match &self {
            ReturnType::Base => ::scandir::ReturnType::Base,
            ReturnType::Ext => ::scandir::ReturnType::Ext,
        }
    }
}

pub fn timeout_from_secs(timeout: Option<f64>) -> PyResult<Option<Duration>> {
    let Some(secs) = timeout else {
        return Ok(None);
    };
    if !secs.is_finite() || secs < 0.0 {
        return Err(PyValueError::new_err(
            "timeout must be a finite, non-negative number",
        ));
    }
    if secs == 0.0 {
        return Ok(None);
    }
    let duration = Duration::try_from_secs_f64(secs)
        .map_err(|_| PyValueError::new_err("timeout is too large"))?;
    if Instant::now().checked_add(duration).is_none() {
        return Err(PyValueError::new_err("timeout is too large"));
    }
    Ok(Some(duration))
}
