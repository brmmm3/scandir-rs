use std::time::UNIX_EPOCH;
use std::collections::HashMap;

use jwalk::WalkDir;
#[cfg(unix)]
use expanduser::expanduser;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use pyo3::PyIterProtocol;
use pyo3::PyObjectProtocol;

#[pyclass]
#[derive(Debug)]
struct DirEntry {
    is_symlink: bool,
    is_dir: bool,
    is_file: bool,
    ctime: f64,
    mtime: f64,
    atime: f64,
    mode: u32,
    ino: u64,
    dev: u64,
    nlink: u64,
    size: u64,
    blksize: u64,
    blocks: u64,
    #[cfg(unix)]
    uid: u32,
    #[cfg(unix)]
    gid: u32,
    #[cfg(unix)]
    rdev: u64,
}

#[pymethods]
impl DirEntry {
    #[new]
    fn __new__(
        obj: &PyRawObject,
        is_symlink: bool,
        is_dir: bool,
        is_file: bool,
        ctime: f64,
        mtime: f64,
        atime: f64,
        mode: u32,
        ino: u64,
        dev: u64,
        nlink: u64,
        size: u64,
        blksize: u64,
        blocks: u64,
        #[cfg(unix)]
        uid: u32,
        #[cfg(unix)]
        gid: u32,
        #[cfg(unix)]
        rdev: u64,
    ) {
        obj.init(DirEntry {
            is_symlink: is_symlink,
            is_dir: is_dir,
            is_file: is_file,
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
        });
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for DirEntry {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
    }
}

#[pyclass]
struct ScanDirIterator {
    iter: Box<dyn Iterator<Item = PyObject> + Send>,
}

#[pyproto]
impl PyIterProtocol for ScanDirIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<ScanDirIterator>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        Ok(slf.iter.next())
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
    let mut dirs: u32 = 0;
    let mut files: u32 = 0;
    let mut slinks: u32 = 0;
    let mut hlinks: u32 = 0;
    let mut devices: u32 = 0;
    let mut pipes: u32 = 0;
    let mut size: u64 = 0;
    let mut usage: u64 = 0;
    let mut errors = Vec::new();

    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        #[cfg(unix)]
        let root_path = expanduser(root_path)?;
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
                        dirs += 1;
                    }
                    if file_type.is_file() {
                        files += 1;
                    }
                    if file_type.is_symlink() {
                        slinks += 1;
                    }
                    if v.metadata_result.is_some() {
                        let metadata_ext = v.ext.as_ref();
                        if metadata_ext.is_some() {
                            let metadata_ext = metadata_ext.unwrap().as_ref().unwrap();
                            if metadata_ext.nlink > 1 {
                                hlinks += 1;
                            }
                            size += metadata_ext.size;
                            #[cfg(unix)]
                            {
                                if metadata_ext.rdev > 0 {
                                    devices += 1;
                                }
                                if (metadata_ext.mode & 4096) != 0 {
                                    pipes += 1;
                                }
                                usage += metadata_ext.blocks * 512;
                            }
                            #[cfg(windows)]
                            {
                                let mut blocks = metadata_ext.size >> 12;
                                if blocks << 12 < metadata_ext.size {
                                    blocks += 1;
                                }
                                usage += blocks << 12;
                            }
                        }
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
    if dirs > 0 {
        pyresult.set_item("dirs", dirs).unwrap();
    }
    if files > 0 {
        pyresult.set_item("files", files).unwrap();
    }
    if slinks > 0 {
        pyresult.set_item("slinks", slinks).unwrap();
    }
    if hlinks > 0 {
        pyresult.set_item("hlinks", hlinks).unwrap();
    }
    if devices > 0 {
        pyresult.set_item("devices", devices).unwrap();
    }
    if pipes > 0 {
        pyresult.set_item("pipes", pipes).unwrap();
    }
    pyresult.set_item("size", size).unwrap();
    pyresult.set_item("usage", usage).unwrap();
    if !errors.is_empty() {
        pyresult.set_item("errors", errors).unwrap();
    }
    Ok(pyresult.into())
}

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

#[pyfunction]
pub fn list(
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

#[pymodule]
fn scandir_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<DirEntry>()?;
    m.add_class::<ScanDirIterator>()?;
    m.add_wrapped(wrap_pyfunction!(count))?;
    m.add_wrapped(wrap_pyfunction!(toc))?;
    m.add_wrapped(wrap_pyfunction!(list))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        println!(
            "{:#?}",
            count(py, "C:/temp", Some(false), Some(false), Some(false)).unwrap()
        );
    }

    #[test]
    fn test_count_skip_hidden() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        count(py, "C:/temp", Some(true), Some(false), Some(false)).unwrap();
    }

    #[test]
    fn test_count_skip_hidden_metadata() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        println!(
            "{:#?}",
            count(py, "C:/temp", Some(true), Some(true), Some(false)).unwrap()
        );
    }

    #[test]
    fn test_count_skip_hidden_metadata_ext() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        count(py, "C:/temp", Some(true), Some(true), Some(true)).unwrap();
    }
}
