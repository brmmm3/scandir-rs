use glob::{MatchOptions, Pattern};

use pyo3::prelude::*;
use pyo3::types::PyTuple;

#[derive(Debug, Clone)]
pub struct Filter {
    pub dir_include: Vec<Pattern>,
    pub dir_exclude: Vec<Pattern>,
    pub file_include: Vec<Pattern>,
    pub file_exclude: Vec<Pattern>,
    pub options: Option<MatchOptions>,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    #[pyo3(get)]
    pub st_ctime: f64,
    #[pyo3(get)]
    pub st_mtime: f64,
    #[pyo3(get)]
    pub st_atime: f64,
}

#[pymethods]
impl DirEntry {
    #[new]
    fn new(
        is_symlink: bool,
        is_dir: bool,
        is_file: bool,
        st_ctime: f64,
        st_mtime: f64,
        st_atime: f64,
    ) -> Self {
        DirEntry {
            is_symlink: is_symlink,
            is_dir: is_dir,
            is_file: is_file,
            st_ctime: st_ctime,
            st_mtime: st_mtime,
            st_atime: st_atime,
        }
    }

    fn is_symlink(&self) -> PyResult<bool> {
        Ok(self.is_symlink.into())
    }

    fn is_dir(&self) -> PyResult<bool> {
        Ok(self.is_dir.into())
    }

    fn is_file(&self) -> PyResult<bool> {
        Ok(self.is_file.into())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct DirEntryExt {
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    #[pyo3(get)]
    pub st_ctime: f64,
    #[pyo3(get)]
    pub st_mtime: f64,
    #[pyo3(get)]
    pub st_atime: f64,
    #[pyo3(get)]
    pub st_mode: u32,
    #[pyo3(get)]
    pub st_ino: u64,
    #[pyo3(get)]
    pub st_dev: u64,
    #[pyo3(get)]
    pub st_nlink: u64,
    #[pyo3(get)]
    pub st_size: u64,
    #[pyo3(get)]
    pub st_blksize: u64,
    #[pyo3(get)]
    pub st_blocks: u64,
    #[pyo3(get)]
    pub st_uid: u32,
    #[pyo3(get)]
    pub st_gid: u32,
    #[pyo3(get)]
    pub st_rdev: u64,
}

#[pymethods]
impl DirEntryExt {
    #[new]
    fn new(
        is_symlink: bool,
        is_dir: bool,
        is_file: bool,
        st_ctime: f64,
        st_mtime: f64,
        st_atime: f64,
        st_mode: u32,
        st_ino: u64,
        st_dev: u64,
        st_nlink: u64,
        st_size: u64,
        st_blksize: u64,
        st_blocks: u64,
        st_uid: u32,
        st_gid: u32,
        st_rdev: u64,
    ) -> Self {
        DirEntryExt {
            is_symlink: is_symlink,
            is_dir: is_dir,
            is_file: is_file,
            st_ctime: st_ctime,
            st_mtime: st_mtime,
            st_atime: st_atime,
            st_mode: st_mode,
            st_ino: st_ino,
            st_dev: st_dev,
            st_nlink: st_nlink,
            st_size: st_size,
            st_blksize: st_blksize,
            st_blocks: st_blocks,
            st_uid: st_uid,
            st_gid: st_gid,
            st_rdev: st_rdev,
        }
    }

    fn is_symlink(&self) -> PyResult<bool> {
        Ok(self.is_symlink.into())
    }

    fn is_dir(&self) -> PyResult<bool> {
        Ok(self.is_dir.into())
    }

    fn is_file(&self) -> PyResult<bool> {
        Ok(self.is_file.into())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct DirEntryFull {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub path: String,
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    #[pyo3(get)]
    pub st_ctime: f64,
    #[pyo3(get)]
    pub st_mtime: f64,
    #[pyo3(get)]
    pub st_atime: f64,
    #[pyo3(get)]
    pub st_mode: u32,
    #[pyo3(get)]
    pub st_ino: u64,
    #[pyo3(get)]
    pub st_dev: u64,
    #[pyo3(get)]
    pub st_nlink: u64,
    #[pyo3(get)]
    pub st_size: u64,
    #[pyo3(get)]
    pub st_blksize: u64,
    #[pyo3(get)]
    pub st_blocks: u64,
    #[pyo3(get)]
    pub st_uid: u32,
    #[pyo3(get)]
    pub st_gid: u32,
    #[pyo3(get)]
    pub st_rdev: u64,
}

#[pymethods]
impl DirEntryFull {
    #[new]
    fn new(
        name: String,
        path: String,
        is_symlink: bool,
        is_dir: bool,
        is_file: bool,
        st_ctime: f64,
        st_mtime: f64,
        st_atime: f64,
        st_mode: u32,
        st_ino: u64,
        st_dev: u64,
        st_nlink: u64,
        st_size: u64,
        st_blksize: u64,
        st_blocks: u64,
        st_uid: u32,
        st_gid: u32,
        st_rdev: u64,
    ) -> Self {
        DirEntryFull {
            name: name,
            path: path,
            is_symlink: is_symlink,
            is_dir: is_dir,
            is_file: is_file,
            st_ctime: st_ctime,
            st_mtime: st_mtime,
            st_atime: st_atime,
            st_mode: st_mode,
            st_ino: st_ino,
            st_dev: st_dev,
            st_nlink: st_nlink,
            st_size: st_size,
            st_blksize: st_blksize,
            st_blocks: st_blocks,
            st_uid: st_uid,
            st_gid: st_gid,
            st_rdev: st_rdev,
        }
    }

    fn is_symlink(&self) -> PyResult<bool> {
        Ok(self.is_symlink.into())
    }

    fn is_dir(&self) -> PyResult<bool> {
        Ok(self.is_dir.into())
    }

    fn is_file(&self) -> PyResult<bool> {
        Ok(self.is_file.into())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
    }
}

#[derive(Debug, Clone)]
pub enum ScandirResult {
    DirEntry(DirEntry),
    DirEntryExt(DirEntryExt),
    DirEntryFull(DirEntryFull),
    Error(String),
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Toc {
    #[pyo3(get)]
    pub dirs: Vec<String>,
    #[pyo3(get)]
    pub files: Vec<String>,
    #[pyo3(get)]
    pub symlinks: Vec<String>,
    #[pyo3(get)]
    pub other: Vec<String>,
    #[pyo3(get)]
    pub errors: Vec<String>,
}

#[pymethods]
impl Toc {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    pub fn clear(&mut self) {
        self.dirs.clear();
        self.files.clear();
        self.symlinks.clear();
        self.other.clear();
        self.errors.clear();
    }
}

impl ToPyObject for Toc {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyTuple::new(
            py,
            &[
                self.dirs.to_object(py),
                self.files.to_object(py),
                self.symlinks.to_object(py),
                self.other.to_object(py),
                self.errors.to_object(py),
            ],
        )
        .into()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct WalkEntry {
    pub path: String,
    pub toc: Toc,
}

#[pymethods]
impl WalkEntry {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

impl ToPyObject for WalkEntry {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyTuple::new(
            py,
            &[
                self.path.to_object(py),
                self.toc.dirs.to_object(py),
                self.toc.files.to_object(py),
            ],
        )
        .into()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct WalkEntryExt {
    pub path: String,
    pub toc: Toc,
}

#[pymethods]
impl WalkEntryExt {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

impl ToPyObject for WalkEntryExt {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyTuple::new(
            py,
            &[
                self.path.to_object(py),
                self.toc.dirs.to_object(py),
                self.toc.files.to_object(py),
                self.toc.symlinks.to_object(py),
                self.toc.other.to_object(py),
                self.toc.errors.to_object(py),
            ],
        )
        .into()
    }
}

#[derive(Debug, Clone)]
pub enum WalkResult {
    Toc(Toc),
    WalkEntry(WalkEntry),
    WalkEntryExt(WalkEntryExt),
}
