use std::thread;

use jwalk::WalkDir;
#[cfg(unix)]
use expanduser::expanduser;

use pyo3::prelude::*;
use pyo3::types::{PyType, PyAny, PyDict};
use pyo3::exceptions::ValueError;
use pyo3::{PyContextProtocol, PyIterProtocol};
use pyo3::wrap_pyfunction;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Statistics {
    pub dirs: u32,
    pub files: u32,
    pub slinks: u32,
    pub hlinks: u32,
    pub devices: u32,
    pub pipes: u32,
    pub size: u64,
    pub usage: u64,
    pub errors: Vec<String>,
}

fn counter(
    root_path: &str,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
    statistics: &mut Statistics,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden.unwrap_or(false))
        .sort(false)
        .preload_metadata(metadata.unwrap_or(false))
        .preload_metadata_ext(metadata_ext.unwrap_or(false))
    {
        match &entry {
            Ok(v) => {
                let file_type = v.file_type_result.as_ref().unwrap();
                if file_type.is_dir() {
                    statistics.dirs += 1;
                }
                if file_type.is_file() {
                    statistics.files += 1;
                }
                if file_type.is_symlink() {
                    statistics.slinks += 1;
                }
                if v.metadata_result.is_some() {
                    let metadata_ext = v.ext.as_ref();
                    if metadata_ext.is_some() {
                        let metadata_ext = metadata_ext.unwrap().as_ref().unwrap();
                        if metadata_ext.nlink > 1 {
                            statistics.hlinks += 1;
                        }
                        statistics.size += metadata_ext.size;
                        #[cfg(unix)]
                        {
                            if metadata_ext.rdev > 0 {
                                statistics.devices += 1;
                            }
                            if (metadata_ext.mode & 4096) != 0 {
                                statistics.pipes += 1;
                            }
                            statistics.usage += metadata_ext.blocks * 512;
                        }
                        #[cfg(windows)]
                        {
                            let mut blocks = metadata_ext.size >> 12;
                            if blocks << 12 < metadata_ext.size {
                                blocks += 1;
                            }
                            statistics.usage += blocks << 12;
                        }
                    }
                }
            }
            Err(e) => statistics.errors.push(e.to_string())  // TODO: Need to fetch failed path from somewhere
        };
    }
}

#[pyfunction]
pub fn count(
    py: Python,
    root_path: &str,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
) -> PyResult<PyObject> {
    let mut statistics = Statistics { 
        dirs: 0,
        files: 0,
        slinks: 0,
        hlinks: 0,
        devices: 0,
        pipes: 0,
        size: 0,
        usage: 0,
        errors: Vec::new(),
    };

    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        counter(root_path, skip_hidden, metadata, metadata_ext, &mut statistics);
        Ok(())
    });
    let pyresult = PyDict::new(py);
    match rc {
        Err(e) => { pyresult.set_item("error", e.to_string()).unwrap();
                    return Ok(pyresult.into())
                  },
        _ => ()
    }
    if statistics.dirs > 0 {
        pyresult.set_item("dirs", statistics.dirs).unwrap();
    }
    if statistics.files > 0 {
        pyresult.set_item("files", statistics.files).unwrap();
    }
    if statistics.slinks > 0 {
        pyresult.set_item("slinks", statistics.slinks).unwrap();
    }
    if statistics.hlinks > 0 {
        pyresult.set_item("hlinks", statistics.hlinks).unwrap();
    }
    if statistics.devices > 0 {
        pyresult.set_item("devices", statistics.devices).unwrap();
    }
    if statistics.pipes > 0 {
        pyresult.set_item("pipes", statistics.pipes).unwrap();
    }
    pyresult.set_item("size", statistics.size).unwrap();
    pyresult.set_item("usage", statistics.usage).unwrap();
    if !statistics.errors.is_empty() {
        pyresult.set_item("errors", statistics.errors).unwrap();
    }
    Ok(pyresult.into())
}

#[pyclass]
#[derive(Debug)]
pub struct Counter {
    count: Count,
}

#[pymethods]
impl Counter {
     #[getter]
     fn statistics(&self) -> PyResult<Statistics> {
        Ok(Statistics { 
            dirs: 0,
            files: 0,
            slinks: 0,
            hlinks: 0,
            devices: 0,
            pipes: 0,
            size: 0,
            usage: 0,
            errors: Vec::new(),
        })
     }
}

#[pyclass]
#[derive(Debug)]
pub struct Count {
    root_path: String,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
    exit_called: bool,
    thr: Option<thread::JoinHandle<()>>,
    statistics: Statistics,
}

#[pymethods]
impl Count {
    #[new]
    fn __new__(
        obj: &PyRawObject,
        root_path: &str,
        skip_hidden: Option<bool>,
        metadata: Option<bool>,
        metadata_ext: Option<bool>,
    ) {
        println!("Count.__new__");
        obj.init(Count {
            root_path: String::from(root_path),
            skip_hidden: skip_hidden,
            metadata: metadata,
            metadata_ext: metadata_ext,
            exit_called: false,
            thr: None,
            statistics: Statistics { 
                dirs: 0,
                files: 0,
                slinks: 0,
                hlinks: 0,
                devices: 0,
                pipes: 0,
                size: 0,
                usage: 0,
                errors: Vec::new(),
            },
        });
    }

    #[getter]
    fn root_path(&self) -> PyResult<String> {
       Ok(String::from(&self.root_path))
    }
}

#[pyproto]
impl<'p> PyIterProtocol for Count {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Count>> {
        slf.thr = Some(thread::spawn(|| {
            counter(slf.root_path.as_ref(), slf.skip_hidden, slf.metadata, slf.metadata_ext,
                    &mut slf.statistics)
        }));
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<i32>> {
        Ok(slf.iter.next())
    }
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for Count {
    fn __enter__(&mut self) -> PyResult<PyObject> {
        println!("__enter__ {:?}", self);
        let gil = GILGuard::acquire();
        self.thr = Some(thread::spawn(|| {
            counter(self.root_path.as_ref(), self.skip_hidden, self.metadata, self.metadata_ext,
                    &mut self.statistics)
        }));
        //let iter = IntoPy::into_py(
        //    Py::new(py, self,
        //    py,
        //);
        Ok(IntoPy::into_py(
            Py::new(gil.python(), *self),
            gil.python(),
        ).unwrap())
        //Ok(PyRefMut::new(gil.python(), *self).unwrap().into())
    }

    fn __exit__(
        &mut self,
        ty: Option<&'p PyType>,
        _value: Option<&'p PyAny>,
        _traceback: Option<&'p PyAny>,
    ) -> PyResult<bool> {
        println!("__exit__ {:?}", self);
        println!("{:?}", ty);
        println!("{:?}", _value);
        println!("{:?}", _traceback);
        self.thr.unwrap().join();
        let gil = GILGuard::acquire();
        self.exit_called = true;
        if ty == Some(gil.python().get_type::<ValueError>()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[pymodule(count)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Count>()?;
    m.add_wrapped(wrap_pyfunction!(count))?;
    Ok(())
}
