use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::PyObjectProtocol;

pub const ITER_TYPE_TOC: u8 = 0;
pub const ITER_TYPE_WALK: u8 = 1;
pub const ITER_TYPE_WALKEXT: u8 = 2;

#[pyclass]
#[derive(Debug, Clone)]
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
    pub uid: u32,
    pub gid: u32,
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
        uid: u32,
        gid: u32,
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
    fn ctime(&self) -> PyResult<f64> {
        Ok(self.ctime.into())
    }

    #[getter]
    fn mtime(&self) -> PyResult<f64> {
        Ok(self.mtime.into())
    }

    #[getter]
    fn atime(&self) -> PyResult<f64> {
        Ok(self.atime.into())
    }

    #[getter]
    fn mode(&self) -> PyResult<u32> {
        Ok(self.mode.into())
    }

    #[getter]
    fn ino(&self) -> PyResult<u64> {
        Ok(self.ino.into())
    }

    #[getter]
    fn dev(&self) -> PyResult<u64> {
        Ok(self.dev.into())
    }

    #[getter]
    fn nlink(&self) -> PyResult<u64> {
        Ok(self.nlink.into())
    }

    #[getter]
    fn size(&self) -> PyResult<u64> {
        Ok(self.size.into())
    }

    #[getter]
    fn blksize(&self) -> PyResult<u64> {
        Ok(self.blksize.into())
    }

    #[getter]
    fn blocks(&self) -> PyResult<u64> {
        Ok(self.blocks.into())
    }

    #[getter]
    fn uid(&self) -> PyResult<u32> {
        Ok(self.uid.into())
    }

    #[getter]
    fn gid(&self) -> PyResult<u32> {
        Ok(self.gid.into())
    }

    #[getter]
    fn rdev(&self) -> PyResult<u64> {
        Ok(self.rdev.into())
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
pub enum IterResult {
    Toc(Toc),
    WalkEntry(WalkEntry),
}
