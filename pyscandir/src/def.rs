use pyo3::prelude::*;
use pyo3::types::PyDict;

use scandir;

#[pyclass]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ReturnType {
    Base,
    Ext,
}

impl ReturnType {
    pub fn from_object(&self) -> scandir::ReturnType {
        match &self {
            ReturnType::Base => scandir::ReturnType::Base,
            ReturnType::Ext => scandir::ReturnType::Ext,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct DirEntry {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub is_symlink: bool,
    #[pyo3(get)]
    pub is_dir: bool,
    #[pyo3(get)]
    pub is_file: bool,
    #[pyo3(get)]
    pub st_ctime: f64,
    #[pyo3(get)]
    pub st_mtime: f64,
    #[pyo3(get)]
    pub st_atime: f64,
}

impl DirEntry {
    pub fn new(entry: &scandir::DirEntry) -> Self {
        DirEntry {
            path: entry.path.clone(),
            is_symlink: entry.is_symlink,
            is_dir: entry.is_dir,
            is_file: entry.is_file,
            st_ctime: entry.st_ctime,
            st_mtime: entry.st_mtime,
            st_atime: entry.st_atime,
        }
    }
}

#[pymethods]
impl DirEntry {
    pub fn as_dict(&self, py: Python) -> PyResult<PyObject> {
        let pydict = PyDict::new(py);
        pydict.set_item("path".to_object(py), self.path.clone())?;
        pydict.set_item("is_symlink".to_object(py), self.is_symlink)?;
        pydict.set_item("is_dir".to_object(py), self.is_dir)?;
        pydict.set_item("is_file".to_object(py), self.is_file)?;
        pydict.set_item("st_ctime".to_object(py), self.st_ctime)?;
        pydict.set_item("st_mtime".to_object(py), self.st_mtime)?;
        pydict.set_item("st_atime".to_object(py), self.st_atime)?;
        Ok(pydict.to_object(py))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct DirEntryExt {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub is_symlink: bool,
    #[pyo3(get)]
    pub is_dir: bool,
    #[pyo3(get)]
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

impl DirEntryExt {
    pub fn new(entry: &scandir::DirEntryExt) -> Self {
        DirEntryExt {
            path: entry.path.clone(),
            is_symlink: entry.is_symlink,
            is_dir: entry.is_dir,
            is_file: entry.is_file,
            st_ctime: entry.st_ctime,
            st_mtime: entry.st_mtime,
            st_atime: entry.st_atime,
            st_mode: entry.st_mode,
            st_ino: entry.st_ino,
            st_dev: entry.st_dev,
            st_nlink: entry.st_nlink,
            st_size: entry.st_size,
            st_blksize: entry.st_blksize,
            st_blocks: entry.st_blocks,
            st_uid: entry.st_uid,
            st_gid: entry.st_gid,
            st_rdev: entry.st_rdev,
        }
    }
}

#[pymethods]
impl DirEntryExt {
    pub fn as_dict(&self, py: Python) -> PyResult<PyObject> {
        let pydict = PyDict::new(py);
        pydict.set_item("path".to_object(py), self.path.clone())?;
        pydict.set_item("is_symlink".to_object(py), self.is_symlink)?;
        pydict.set_item("is_dir".to_object(py), self.is_dir)?;
        pydict.set_item("is_file".to_object(py), self.is_file)?;
        pydict.set_item("st_ctime".to_object(py), self.st_ctime)?;
        pydict.set_item("st_mtime".to_object(py), self.st_mtime)?;
        pydict.set_item("st_atime".to_object(py), self.st_atime)?;
        pydict.set_item("st_mode".to_object(py), self.st_mode)?;
        pydict.set_item("st_ino".to_object(py), self.st_ino)?;
        pydict.set_item("st_dev".to_object(py), self.st_dev)?;
        pydict.set_item("st_nlink".to_object(py), self.st_nlink)?;
        pydict.set_item("st_size".to_object(py), self.st_size)?;
        pydict.set_item("st_blksize".to_object(py), self.st_blksize)?;
        pydict.set_item("st_blocks".to_object(py), self.st_blocks)?;
        pydict.set_item("st_uid".to_object(py), self.st_uid)?;
        pydict.set_item("st_gid".to_object(py), self.st_gid)?;
        pydict.set_item("st_rdev".to_object(py), self.st_rdev)?;
        Ok(pydict.to_object(py))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
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

impl Toc {
    pub fn new(toc: Option<scandir::Toc>) -> Self {
        if let Some(toc) = toc {
            Toc {
                dirs: toc.dirs,
                files: toc.files,
                symlinks: toc.symlinks,
                other: toc.other,
                errors: toc.errors,
            }
        } else {
            Toc {
                dirs: Vec::new(),
                files: Vec::new(),
                symlinks: Vec::new(),
                other: Vec::new(),
                errors: Vec::new(),
            }
        }
    }
}

impl Toc {
    pub fn extend(&mut self, toc: &scandir::Toc) {
        self.dirs.extend_from_slice(&toc.dirs);
        self.files.extend_from_slice(&toc.files);
        self.symlinks.extend_from_slice(&toc.symlinks);
        self.other.extend_from_slice(&toc.other);
        self.errors.extend_from_slice(&toc.errors);
    }
}

#[pymethods]
impl Toc {
    pub fn as_dict(&self, py: Python) -> PyResult<PyObject> {
        let pydict = PyDict::new(py);
        pydict.set_item("dirs".to_object(py), self.dirs.clone())?;
        pydict.set_item("files".to_object(py), self.files.clone())?;
        pydict.set_item("symlinks".to_object(py), self.symlinks.clone())?;
        pydict.set_item("other".to_object(py), self.other.clone())?;
        pydict.set_item("errors".to_object(py), self.errors.clone())?;
        Ok(pydict.to_object(py))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Statistics {
    #[pyo3(get)]
    pub dirs: i32,
    #[pyo3(get)]
    pub files: i32,
    #[pyo3(get)]
    pub slinks: i32,
    #[pyo3(get)]
    pub hlinks: i32,
    #[pyo3(get)]
    pub devices: i32,
    #[pyo3(get)]
    pub pipes: i32,
    #[pyo3(get)]
    pub size: u64,
    #[pyo3(get)]
    pub usage: u64,
    #[pyo3(get)]
    pub errors: Vec<String>,
    #[pyo3(get)]
    pub duration: f64,
}

impl Statistics {
    pub fn new(statistics: Option<scandir::Statistics>) -> Self {
        if let Some(statistics) = statistics {
            Statistics {
                dirs: statistics.dirs,
                files: statistics.files,
                slinks: statistics.slinks,
                hlinks: statistics.hlinks,
                devices: statistics.devices,
                pipes: statistics.pipes,
                size: statistics.size,
                usage: statistics.usage,
                errors: statistics.errors.clone(),
                duration: statistics.duration,
            }
        } else {
            Statistics {
                dirs: 0,
                files: 0,
                slinks: 0,
                hlinks: 0,
                devices: 0,
                pipes: 0,
                size: 0,
                usage: 0,
                errors: Vec::new(),
                duration: 0.0,
            }
        }
    }
}

#[pymethods]
impl Statistics {
    pub fn as_dict(&self, duration: Option<bool>, py: Python) -> PyResult<PyObject> {
        let pyresult = PyDict::new(py);
        if self.dirs > 0 {
            pyresult.set_item("dirs", self.dirs).unwrap();
        }
        if self.files > 0 {
            pyresult.set_item("files", self.files).unwrap();
        }
        if self.slinks > 0 {
            pyresult.set_item("slinks", self.slinks).unwrap();
        }
        if self.hlinks > 0 {
            pyresult.set_item("hlinks", self.hlinks).unwrap();
        }
        if self.devices > 0 {
            pyresult.set_item("devices", self.devices).unwrap();
        }
        if self.pipes > 0 {
            pyresult.set_item("pipes", self.pipes).unwrap();
        }
        if self.size > 0 {
            pyresult.set_item("size", self.size).unwrap();
        }
        if self.usage > 0 {
            pyresult.set_item("usage", self.usage).unwrap();
        }
        if !self.errors.is_empty() {
            pyresult.set_item("errors", self.errors.to_vec()).unwrap();
        }
        if duration.unwrap_or(false) == true {
            pyresult.set_item("duration", self.duration).unwrap();
        }
        Ok(pyresult.to_object(py))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
