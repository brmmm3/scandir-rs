use std::iter;

use jwalk::WalkDir;
#[cfg(unix)]
use expanduser::expanduser;

use pyo3::prelude::*;
use pyo3::types::{PyType, PyAny, PyDict};
use pyo3::exceptions::ValueError;
use pyo3::{PyContextProtocol, PyIterProtocol};
use pyo3::wrap_pyfunction;

#[pyfunction]
pub fn toc(
    py: Python,
    root_path: &str,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
) -> PyResult<PyObject> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    let mut symlinks = Vec::new();
    let mut unknown = Vec::new();
    let mut errors = Vec::new();

    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        #[cfg(unix)]
        let root_path = expanduser(root_path)?;
        for entry in WalkDir::new(root_path)
            .skip_hidden(skip_hidden.unwrap_or(false))
            .sort(sorted.unwrap_or(false))
        {
            match &entry {
                Ok(v) => {
                    let file_type = v.file_type_result.as_ref().unwrap();
                    let mut key = v.parent_path.to_path_buf();
                    key.push(v.file_name.clone().into_string().unwrap());
                    if file_type.is_symlink() {
                        symlinks.push(key.to_str().unwrap().to_string());
                    } else if file_type.is_dir() {
                        dirs.push(key.to_str().unwrap().to_string());
                    } else if file_type.is_file() {
                        files.push(key.to_str().unwrap().to_string());
                    } else {
                        unknown.push(key.to_str().unwrap().to_string());
                    }
                }
                Err(e) => errors.push(e.to_string())  // TODO: Need to fetch failed path from somewhere
            };
        }
        Ok(())
    });
    let pyresult = PyDict::new(py);
    match rc {
        Err(e) => { pyresult.set_item("error", e.to_string()).unwrap();
                    return Ok(pyresult.into())
                  },
        _ => ()
    }
    if !dirs.is_empty() {
        pyresult.set_item("dirs", dirs).unwrap();
    }
    if !files.is_empty() {
        pyresult.set_item("files", files).unwrap();
    }
    if !symlinks.is_empty() {
        pyresult.set_item("symlinks", symlinks).unwrap();
    }
    if !unknown.is_empty() {
        pyresult.set_item("unknown", unknown).unwrap();
    }
    if !errors.is_empty() {
        pyresult.set_item("errors", errors).unwrap();
    }
    Ok(pyresult.into())
}

#[pyclass]
pub struct WalkIter {
    iter: Box<dyn iter::Iterator<Item = i32> + Send>,
}

#[pymethods]
impl WalkIter {
    #[new]
    fn __new__(
        obj: &PyRawObject,
    ) {
        obj.init(WalkIter { iter: Box::new(5..8),
        });
    }
}

#[pyproto]
impl<'p> PyIterProtocol for WalkIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<WalkIter>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<i32>> {
        Ok(slf.iter.next())
    }
}

#[pyclass]
pub struct Walk {
    exit_called: bool,
}

#[pymethods]
impl Walk {
    #[new]
    fn __new__(
        obj: &PyRawObject,
    ) {
        obj.init(Walk { exit_called: false,
        });
    }
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for Walk {
    fn __enter__(&mut self) -> PyResult<i32> {
        Ok(42)
    }

    fn __exit__(
        &mut self,
        ty: Option<&'p PyType>,
        _value: Option<&'p PyAny>,
        _traceback: Option<&'p PyAny>,
    ) -> PyResult<bool> {
        let gil = GILGuard::acquire();
        self.exit_called = true;
        if ty == Some(gil.python().get_type::<ValueError>()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[pymodule(walk)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Walk>()?;
    m.add_wrapped(wrap_pyfunction!(toc))?;
    Ok(())
}
