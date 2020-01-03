use pyo3::prelude::*;
use pyo3::PyObjectProtocol;

#[pyclass]
#[derive(Debug)]
pub struct DirEntry {
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    pub ctime: f64,
    pub mtime: f64,
    pub atime: f64,
    pub mode: u32,
    pub ino: u64,
    pub dev: u64,
    pub nlink: u64,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    #[cfg(unix)]
    pub uid: u32,
    #[cfg(unix)]
    pub gid: u32,
    #[cfg(unix)]
    pub rdev: u64,
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
