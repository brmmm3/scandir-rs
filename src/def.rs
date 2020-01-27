use glob::{MatchOptions, Pattern};

use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::PyObjectProtocol;

/// Read only file- and directory names without metadata
pub const RETURN_TYPE_FAST: u8 = 0;
/// Read also basic metadata
pub const RETURN_TYPE_BASE: u8 = 1;
/// Read also extended metadata
pub const RETURN_TYPE_EXT: u8 = 2;
/// Also provide relative path and filename in result
pub const RETURN_TYPE_FULL: u8 = 3;
pub const RETURN_TYPE_WALK: u8 = 4;

#[pyclass]
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    pub ctime: f64,
    pub mtime: f64,
    pub atime: f64,
}

#[pymethods]
impl DirEntry {
    #[new]
    fn new(
        obj: &PyRawObject,
        is_symlink: bool,
        is_dir: bool,
        is_file: bool,
        ctime: f64,
        mtime: f64,
        atime: f64,
    ) {
        obj.init(DirEntry {
            is_symlink: is_symlink,
            is_dir: is_dir,
            is_file: is_file,
            ctime: ctime,
            mtime: mtime,
            atime: atime,
        });
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

    #[getter]
    fn st_ctime(&self) -> PyResult<f64> {
        Ok(self.ctime.into())
    }

    #[getter]
    fn st_mtime(&self) -> PyResult<f64> {
        Ok(self.mtime.into())
    }

    #[getter]
    fn st_atime(&self) -> PyResult<f64> {
        Ok(self.atime.into())
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for DirEntry {
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
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
}

#[pymethods]
impl DirEntryExt {
    #[new]
    fn new(
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
        uid: u32,
        gid: u32,
        rdev: u64,
    ) {
        obj.init(DirEntryExt {
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
            uid: uid,
            gid: gid,
            rdev: rdev,
        });
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

    #[getter]
    fn st_ctime(&self) -> PyResult<f64> {
        Ok(self.ctime.into())
    }

    #[getter]
    fn st_mtime(&self) -> PyResult<f64> {
        Ok(self.mtime.into())
    }

    #[getter]
    fn st_atime(&self) -> PyResult<f64> {
        Ok(self.atime.into())
    }

    #[getter]
    fn st_mode(&self) -> PyResult<u32> {
        Ok(self.mode.into())
    }

    #[getter]
    fn st_ino(&self) -> PyResult<u64> {
        Ok(self.ino.into())
    }

    #[getter]
    fn st_dev(&self) -> PyResult<u64> {
        Ok(self.dev.into())
    }

    #[getter]
    fn st_nlink(&self) -> PyResult<u64> {
        Ok(self.nlink.into())
    }

    #[getter]
    fn st_size(&self) -> PyResult<u64> {
        Ok(self.size.into())
    }

    #[getter]
    fn st_blksize(&self) -> PyResult<u64> {
        Ok(self.blksize.into())
    }

    #[getter]
    fn st_blocks(&self) -> PyResult<u64> {
        Ok(self.blocks.into())
    }

    #[getter]
    fn st_uid(&self) -> PyResult<u32> {
        Ok(self.uid.into())
    }

    #[getter]
    fn st_gid(&self) -> PyResult<u32> {
        Ok(self.gid.into())
    }

    #[getter]
    fn st_rdev(&self) -> PyResult<u64> {
        Ok(self.rdev.into())
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for DirEntryExt {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct DirEntryFull {
    pub name: String,
    pub path: String,
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
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
}

#[pymethods]
impl DirEntryFull {
    #[new]
    fn new(
        obj: &PyRawObject,
        name: String,
        path: String,
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
        uid: u32,
        gid: u32,
        rdev: u64,
    ) {
        obj.init(DirEntryFull {
            name: name,
            path: path,
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
            uid: uid,
            gid: gid,
            rdev: rdev,
        });
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.name.clone().into())
    }

    #[getter]
    fn path(&self) -> PyResult<String> {
        Ok(self.path.clone().into())
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

    #[getter]
    fn st_ctime(&self) -> PyResult<f64> {
        Ok(self.ctime.into())
    }

    #[getter]
    fn st_mtime(&self) -> PyResult<f64> {
        Ok(self.mtime.into())
    }

    #[getter]
    fn st_atime(&self) -> PyResult<f64> {
        Ok(self.atime.into())
    }

    #[getter]
    fn st_mode(&self) -> PyResult<u32> {
        Ok(self.mode.into())
    }

    #[getter]
    fn st_ino(&self) -> PyResult<u64> {
        Ok(self.ino.into())
    }

    #[getter]
    fn st_dev(&self) -> PyResult<u64> {
        Ok(self.dev.into())
    }

    #[getter]
    fn st_nlink(&self) -> PyResult<u64> {
        Ok(self.nlink.into())
    }

    #[getter]
    fn st_size(&self) -> PyResult<u64> {
        Ok(self.size.into())
    }

    #[getter]
    fn st_blksize(&self) -> PyResult<u64> {
        Ok(self.blksize.into())
    }

    #[getter]
    fn st_blocks(&self) -> PyResult<u64> {
        Ok(self.blocks.into())
    }

    #[getter]
    fn st_uid(&self) -> PyResult<u32> {
        Ok(self.uid.into())
    }

    #[getter]
    fn st_gid(&self) -> PyResult<u32> {
        Ok(self.gid.into())
    }

    #[getter]
    fn st_rdev(&self) -> PyResult<u64> {
        Ok(self.rdev.into())
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for DirEntryFull {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
    }
}

#[derive(Debug, Clone)]
pub enum ScandirResult {
    DirEntry(DirEntry),
    DirEntryExt(DirEntryExt),
    DirEntryFull(DirEntryFull),
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Toc {
    pub dirs: Vec<String>,
    pub files: Vec<String>,
    pub symlinks: Vec<String>,
    pub other: Vec<String>,
    pub errors: Vec<String>,
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Toc {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pymethods]
impl Toc {
    #[getter]
    fn dirs(&self) -> PyResult<Vec<String>> {
        Ok(self.dirs.to_vec())
    }

    #[getter]
    fn files(&self) -> PyResult<Vec<String>> {
        Ok(self.files.to_vec())
    }

    #[getter]
    fn symlinks(&self) -> PyResult<Vec<String>> {
        Ok(self.symlinks.to_vec())
    }

    #[getter]
    fn other(&self) -> PyResult<Vec<String>> {
        Ok(self.other.to_vec())
    }

    #[getter]
    fn errors(&self) -> PyResult<Vec<String>> {
        Ok(self.errors.to_vec())
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

#[pyproto]
impl pyo3::class::PyObjectProtocol for WalkEntry {
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
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub dir_include: Vec<Pattern>,
    pub dir_exclude: Vec<Pattern>,
    pub file_include: Vec<Pattern>,
    pub file_exclude: Vec<Pattern>,
    pub options: Option<MatchOptions>,
}
