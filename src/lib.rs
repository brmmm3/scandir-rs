use std::time::UNIX_EPOCH;

use jwalk::{WalkDir};

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use pyo3::PyObjectProtocol;
use pyo3::PyIterProtocol;

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
    fn __new__(obj: &PyRawObject, is_symlink: bool, is_dir: bool, is_file: bool,
               ctime: f64, mtime: f64, atime: f64,
               mode: u32, ino: u64, dev: u64, nlink: u64, uid: u32, gid: u32, size: u64,
               rdev: u64, blksize: u64, blocks: u64) {
        obj.init(
            DirEntry {
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
                blocks: blocks
            }
        );
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
    fn __new__(obj: &PyRawObject, dirs: u32, files: u32, slinks: u32, hlinks: u32,
               devices: u32, pipes: u32, size: u64, usage: u64, errors: u32) {
        obj.init(
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
            }
        );
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for Count {

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
    }

}

#[pyfunction]
pub fn list(py: Python, root_path: &str, skip_hidden: Option<bool>,
            metadata: Option<bool>, metadata_ext: Option<bool>) -> PyResult<PyObject> {
    let mut entries: Vec<_> = Vec::new();
    let result = PyDict::new(py);

    for entry in WalkDir::new(root_path).skip_hidden(skip_hidden.unwrap_or(false))
                                        .sort(true)
                                        .preload_metadata(metadata.unwrap_or(false))
                                        .preload_metadata_ext(metadata_ext.unwrap_or(false)) {
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
                let mut blksize: u64 = 0;
                let mut blocks: u64 = 0;
                if v.metadata_result.is_some() {
                    let metadata = v.metadata_result.as_ref().unwrap().as_ref().unwrap();
                    let duration = metadata.created().unwrap().duration_since(UNIX_EPOCH).unwrap();
                    ctime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
                    let duration = metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap();
                    mtime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
                    let duration = metadata.accessed().unwrap().duration_since(UNIX_EPOCH).unwrap();
                    atime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
                    let metadata_ext = v.ext.as_ref();
                    if metadata_ext.is_some() {
                        let metadata_ext = metadata_ext.unwrap().as_ref().unwrap();
                        mode = metadata_ext.mode;
                        ino = metadata_ext.ino;
                        dev = metadata_ext.dev;
                        nlink = metadata_ext.nlink;
                        uid = metadata_ext.uid;
                        gid = metadata_ext.gid;
                        size = metadata_ext.size;
                        rdev = metadata_ext.rdev;
                        blksize = metadata_ext.blksize;
                        blocks = metadata_ext.blocks;
                    }
                }
                let mut key = v.parent_path.to_path_buf();
                key.push(v.file_name.clone().into_string().unwrap());
                let entry = PyRef::new(py, DirEntry {
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
                                                blocks: blocks }).unwrap();
                result.set_item(key.to_str(), entry).unwrap();
            },
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

    println!("{:#?}", entries);
    Ok(result.into())
}


#[pyfunction]
pub fn count(py: Python, root_path: &str, skip_hidden: Option<bool>,
             metadata: Option<bool>, metadata_ext: Option<bool>) -> PyResult<PyObject> {
    let mut dirs: u32 = 0;
    let mut files: u32 = 0;
    let mut slinks: u32 = 0;
    let mut hlinks: u32 = 0;
    let mut devices: u32 = 0;
    let mut pipes: u32 = 0;
    let mut size: u64 = 0;
    let mut usage: u64 = 0;
    let mut errors: u32 = 0;
            
    for entry in WalkDir::new(root_path).skip_hidden(skip_hidden.unwrap_or(false))
                                        .sort(true)
                                        .preload_metadata(metadata.unwrap_or(false))
                                        .preload_metadata_ext(metadata_ext.unwrap_or(false)) {
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
                        if metadata_ext.rdev > 0 {
                            devices += 1;
                        }
                        if (metadata_ext.mode & 4096) != 0 {
                            pipes += 1;
                        }
                        size += metadata_ext.size;
                        usage += metadata_ext.blocks * 512;
                    }
                }
            },
            Err(e) => {
                println!("encountered an error: {}", e);
                errors += 1;
            }
        };
    }
    let obj = PyRef::new(py, Count {
        dirs: dirs,
        files: files,
        slinks: slinks,
        hlinks: hlinks,
        devices: devices,
        pipes: pipes,
        size: size,
        usage: usage,
        errors: errors,
    }).unwrap();

    let result = PyDict::new(py);
    result.set_item(root_path, obj).unwrap();
    Ok(result.into())
}


#[pymodule]
fn pyjwalk(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<DirEntry>()?;
    m.add_class::<ScanDirIterator>()?;
    m.add_wrapped(wrap_pyfunction!(list))?;
    m.add_wrapped(wrap_pyfunction!(count))?;
    Ok(())
}

