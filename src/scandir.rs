use std::iter;
use std::time::UNIX_EPOCH;
use std::collections::HashMap;

use jwalk::WalkDir;
#[cfg(unix)]
use expanduser::expanduser;

use pyo3::prelude::*;
use pyo3::types::{PyType, PyAny, PyDict};
use pyo3::exceptions::ValueError;
use pyo3::{PyContextProtocol, PyIterProtocol};
use pyo3::wrap_pyfunction;

use crate::def::*;

#[pyfunction]
pub fn entries(
    py: Python,
    root_path: &str,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
) -> PyResult<PyObject> {
    let mut result = HashMap::new();
    let mut errors = Vec::new();

    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        #[cfg(unix)]
        let root_path = expanduser(root_path)?;
        for entry in WalkDir::new(root_path)
            .skip_hidden(skip_hidden.unwrap_or(false))
            .sort(sorted.unwrap_or(false))
            .preload_metadata(metadata.unwrap_or(false))
            .preload_metadata_ext(metadata_ext.unwrap_or(false))
        {
            match &entry {
                Ok(v) => {
                    let file_type = v.file_type_result.as_ref().unwrap();
                    let mut ctime: f64 = 0.0;
                    let mut mtime: f64 = 0.0;
                    let mut atime: f64 = 0.0;
                    let mut mode: u32 = 0;
                    let mut ino: u64 = 0;
                    let mut dev: u64 = 0;
                    let mut nlink: u64 = 0;
                    let mut size: u64 = 0;
                    let mut blksize: u64 = 4096;
                    let mut blocks: u64 = 0;
                    #[cfg(unix)]
                    let mut uid: u32 = 0;
                    #[cfg(unix)]
                    let mut gid: u32 = 0;
                    #[cfg(unix)]
                    let mut rdev: u64 = 0;
                    if v.metadata_result.is_some() {
                        let metadata = v.metadata_result.as_ref().unwrap().as_ref().unwrap();
                        let duration = metadata
                            .created()
                            .unwrap()
                            .duration_since(UNIX_EPOCH)
                            .unwrap();
                        ctime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
                        let duration = metadata
                            .modified()
                            .unwrap()
                            .duration_since(UNIX_EPOCH)
                            .unwrap();
                        mtime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
                        let duration = metadata
                            .accessed()
                            .unwrap()
                            .duration_since(UNIX_EPOCH)
                            .unwrap();
                        atime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
                        let metadata_ext = v.ext.as_ref();
                        if metadata_ext.is_some() {
                            let metadata_ext = metadata_ext.unwrap().as_ref().unwrap();
                            mode = metadata_ext.mode;
                            ino = metadata_ext.ino;
                            dev = metadata_ext.dev as u64;
                            nlink = metadata_ext.nlink as u64;
                            size = metadata_ext.size;
                            #[cfg(unix)]
                            {
                                blksize = metadata_ext.blksize;
                                blocks = metadata_ext.blocks;
                                uid = metadata_ext.uid;
                                gid = metadata_ext.gid;
                                rdev = metadata_ext.rdev;
                            }
                            #[cfg(windows)]
                            {
                                blksize = 4096;
                                blocks = size >> 12;
                                if blocks << 12 < size {
                                    blocks += 1;
                                }
                            }
                        }
                    }
                    let mut key = v.parent_path.to_path_buf();
                    key.push(v.file_name.clone().into_string().unwrap());
                    let entry = DirEntry {
                            is_symlink: file_type.is_symlink(),
                            is_dir: file_type.is_dir(),
                            is_file: file_type.is_file(),
                            ctime: ctime,
                            mtime: mtime,
                            atime: atime,
                            mode: mode,
                            ino: ino,
                            dev: dev,
                            nlink: nlink,
                            size: size,
                            blksize: blksize,
                            blocks: blocks,
                            #[cfg(unix)]
                            uid: uid,
                            #[cfg(unix)]
                            gid: gid,
                            #[cfg(unix)]
                            rdev: rdev,
                        };
                    result.insert(key.to_str().unwrap().to_string(), entry);
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
    for (k, v) in result.drain() {
        pyresult.set_item(k, PyRef::new(py, v).unwrap())?;
    }
    pyresult.set_item(py.None(), errors)?;
    Ok(pyresult.into())
}

#[pyclass]
pub struct ScandirIter {
    iter: Box<dyn iter::Iterator<Item = i32> + Send>,
}

#[pymethods]
impl ScandirIter {
    #[new]
    fn __new__(
        obj: &PyRawObject,
    ) {
        obj.init(ScandirIter { iter: Box::new(5..8),
        });
    }
}

#[pyproto]
impl<'p> PyIterProtocol for ScandirIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<ScandirIter>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<i32>> {
        Ok(slf.iter.next())
    }
}

#[pyclass]
pub struct Scandir {
    exit_called: bool,
}

#[pymethods]
impl Scandir {
    #[new]
    fn __new__(
        obj: &PyRawObject,
    ) {
        obj.init(Scandir { exit_called: false,
        });
    }
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for Scandir {
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

#[pymodule(scandir)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Scandir>()?;
    m.add_wrapped(wrap_pyfunction!(entries))?;
    Ok(())
}
