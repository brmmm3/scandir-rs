use std::time::UNIX_EPOCH;
use std::cmp;

use jwalk::WalkDir;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
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
    uid: u32,
    gid: u32,
    size: u64,
    rdev: u64,
    blksize: u64,
    blocks: u64,
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
        uid: u32,
        gid: u32,
        size: u64,
        rdev: u64,
        blksize: u64,
        blocks: u64,
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
            uid: uid,
            gid: gid,
            size: size,
            rdev: rdev,
            blksize: blksize,
            blocks: blocks,
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

#[pyclass]
#[derive(Debug)]
struct Count {
    dirs: u32,
    files: u32,
    slinks: u32,
    hlinks: u32,
    devices: u32,
    pipes: u32,
    size: u64,
    usage: u64,
    errors: u32,
}

#[pymethods]
impl Count {
    #[new]
    fn __new__(
        obj: &PyRawObject,
        dirs: u32,
        files: u32,
        slinks: u32,
        hlinks: u32,
        devices: u32,
        pipes: u32,
        size: u64,
        usage: u64,
        errors: u32,
    ) {
        obj.init(Count {
            dirs: dirs,
            files: files,
            slinks: slinks,
            hlinks: hlinks,
            devices: devices,
            pipes: pipes,
            size: size,
            usage: usage,
            errors: errors,
        });
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for Count {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
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
    let mut errors: u32 = 0;

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
            Err(e) => {
                println!("encountered an error: {}", e);
                errors += 1;
            }
        };
    }
    let obj = PyRef::new(
        py,
        Count {
            dirs: dirs,
            files: files,
            slinks: slinks,
            hlinks: hlinks,
            devices: devices,
            pipes: pipes,
            size: size,
            usage: usage,
            errors: errors,
        },
    )
    .unwrap();

    let result = PyDict::new(py);
    result.set_item(root_path, obj).unwrap();
    Ok(result.into())
}

#[pyfunction]
pub fn toc(
    py: Python,
    root_path: &str,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
) -> PyResult<PyObject> {
    let dirs = PyList::empty(py);
    let files = PyList::empty(py);
    let symlinks = PyList::empty(py);
    let unknown = PyList::empty(py);
    let errors = PyDict::new(py);

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
                    symlinks.append(key.to_str()).unwrap();
                } else if file_type.is_dir() {
                    dirs.append(key.to_str()).unwrap();
                } else if file_type.is_file() {
                    files.append(key.to_str()).unwrap();
                } else {
                    unknown.append(key.to_str()).unwrap();
                }
            }
            Err(e) => {
                println!("encountered an error: {}", e);
                let v = entry.as_ref().unwrap();
                let mut key = v.parent_path.to_path_buf();
                key.push(v.file_name.clone().into_string().unwrap());
                errors.set_item(key.to_str(), e.to_string()).unwrap();
            }
        };
    }

    Ok(PyTuple::new(
        py,
        &[
            dirs.to_object(py),
            files.to_object(py),
            symlinks.to_object(py),
            unknown.to_object(py),
            errors.to_object(py),
        ],
    )
    .into())
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
    let mut entries: Vec<_> = Vec::new();
    let result = PyDict::new(py);

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
                let mut uid: u32 = 0;
                let mut gid: u32 = 0;
                let mut size: u64 = 0;
                let mut rdev: u64 = 0;
                let mut blksize: u64 = 4096;
                let mut blocks: u64 = 0;
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
                            uid = metadata_ext.uid;
                            gid = metadata_ext.gid;
                            rdev = metadata_ext.rdev;
                            blksize = metadata_ext.blksize;
                            blocks = metadata_ext.blocks;
                        }
                        #[cfg(windows)]
                        {
                            blocks = size >> 12;
                            if blocks << 12 < size {
                                blocks += 1;
                            }
                        }
                    }
                }
                let mut key = v.parent_path.to_path_buf();
                key.push(v.file_name.clone().into_string().unwrap());
                let entry = PyRef::new(
                    py,
                    DirEntry {
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
                        uid: uid,
                        gid: gid,
                        size: size,
                        rdev: rdev,
                        blksize: blksize,
                        blocks: blocks,
                    },
                )
                .unwrap();
                result.set_item(key.to_str(), entry).unwrap();
            }
            Err(e) => {
                println!("encountered an error: {}", e);
                let v = entry.as_ref().unwrap();
                let mut key = v.parent_path.to_path_buf();
                key.push(v.file_name.clone().into_string().unwrap());
                result.set_item(key.to_str(), e.to_string()).unwrap();
            }
        };
        entries.push(entry);
    }

    for i in 0..cmp::min(entries.len(), 3) {
        println!("{:#?}", entries[i]);
    }
    Ok(result.into())
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
