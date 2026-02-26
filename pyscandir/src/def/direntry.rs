use std::time::SystemTime;

#[cfg(any(feature = "speedy", feature = "bincode", feature = "json"))]
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
#[cfg(any(feature = "speedy", feature = "bincode"))]
use pyo3::types::PyBytes;
use pyo3::types::PyDict;

#[cfg(feature = "speedy")]
use speedy::Writable;

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct DirEntry(scandir::DirEntry);

impl DirEntry {
    pub fn from(entry: &scandir::DirEntry) -> Self {
        DirEntry(entry.clone())
    }
}

#[pymethods]
impl DirEntry {
    #[getter]
    fn path(&self) -> String {
        self.0.path.clone()
    }

    #[getter]
    fn is_symlink(&self) -> bool {
        self.0.is_symlink
    }

    #[getter]
    fn is_dir(&self) -> bool {
        self.0.is_dir
    }

    #[getter]
    fn is_file(&self) -> bool {
        self.0.is_file
    }

    #[getter]
    fn st_ctime(&self) -> Option<SystemTime> {
        self.0.st_ctime
    }

    #[getter]
    fn st_mtime(&self) -> Option<SystemTime> {
        self.0.st_mtime
    }

    #[getter]
    fn st_atime(&self) -> Option<SystemTime> {
        self.0.st_atime
    }

    #[getter]
    fn st_size(&self) -> u64 {
        self.0.st_size
    }

    #[getter]
    fn ctime(&self) -> f64 {
        self.0.ctime()
    }

    #[getter]
    fn mtime(&self) -> f64 {
        self.0.mtime()
    }

    #[getter]
    fn atime(&self) -> f64 {
        self.0.atime()
    }

    fn as_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let pydict = PyDict::new(py);
        pydict.set_item("path", self.0.path.clone())?;
        pydict.set_item("is_symlink", self.0.is_symlink)?;
        pydict.set_item("is_dir", self.0.is_dir)?;
        pydict.set_item("is_file", self.0.is_file)?;
        pydict.set_item("st_ctime", self.0.st_ctime)?;
        pydict.set_item("st_mtime", self.0.st_mtime)?;
        pydict.set_item("st_atime", self.0.st_atime)?;
        pydict.set_item("st_size", self.0.st_size)?;
        Ok(pydict.into_any().unbind())
    }

    #[cfg(feature = "speedy")]
    fn to_speedy(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.0.write_to_vec() {
            Ok(v) => Ok(PyBytes::new_with(py, v.len(), |b| {
                b.copy_from_slice(&v);
                Ok(())
            })?
            .into()),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "bincode")]
    fn to_bincode(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.0.to_vec() {
            Ok(v) => Ok(PyBytes::new_with(py, v.len(), |b| {
                b.copy_from_slice(&v);
                Ok(())
            })?
            .into()),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "json")]
    fn to_json(&self) -> PyResult<String> {
        self.0
            .to_json()
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct DirEntryExt(scandir::DirEntryExt);

impl DirEntryExt {
    pub fn from(entry: &scandir::DirEntryExt) -> Self {
        DirEntryExt(entry.clone())
    }
}

#[pymethods]
impl DirEntryExt {
    #[getter]
    fn path(&self) -> String {
        self.0.path.clone()
    }

    #[getter]
    fn is_symlink(&self) -> bool {
        self.0.is_symlink
    }

    #[getter]
    fn is_dir(&self) -> bool {
        self.0.is_dir
    }

    #[getter]
    fn is_file(&self) -> bool {
        self.0.is_file
    }

    #[getter]
    fn st_ctime(&self) -> Option<SystemTime> {
        self.0.st_ctime
    }

    #[getter]
    fn st_mtime(&self) -> Option<SystemTime> {
        self.0.st_mtime
    }

    #[getter]
    fn st_atime(&self) -> Option<SystemTime> {
        self.0.st_atime
    }

    #[getter]
    fn st_size(&self) -> u64 {
        self.0.st_size
    }

    #[getter]
    fn st_blksize(&self) -> u64 {
        self.0.st_blksize
    }

    #[getter]
    fn st_blocks(&self) -> u64 {
        self.0.st_blocks
    }

    #[getter]
    fn st_mode(&self) -> u32 {
        self.0.st_mode
    }

    #[getter]
    fn st_nlink(&self) -> u64 {
        self.0.st_nlink
    }

    #[getter]
    fn st_uid(&self) -> u32 {
        self.0.st_uid
    }

    #[getter]
    fn st_gid(&self) -> u32 {
        self.0.st_gid
    }

    #[getter]
    fn st_ino(&self) -> u64 {
        self.0.st_ino
    }

    #[getter]
    fn st_dev(&self) -> u64 {
        self.0.st_dev
    }

    #[getter]
    fn st_rdev(&self) -> u64 {
        self.0.st_rdev
    }

    #[getter]
    fn ctime(&self) -> f64 {
        self.0.ctime()
    }

    #[getter]
    fn mtime(&self) -> f64 {
        self.0.mtime()
    }

    #[getter]
    fn atime(&self) -> f64 {
        self.0.atime()
    }

    fn as_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let pydict = PyDict::new(py);
        pydict.set_item("path", self.0.path.clone())?;
        pydict.set_item("is_symlink", self.0.is_symlink)?;
        pydict.set_item("is_dir", self.0.is_dir)?;
        pydict.set_item("is_file", self.0.is_file)?;
        pydict.set_item("st_ctime", self.0.st_ctime)?;
        pydict.set_item("st_mtime", self.0.st_mtime)?;
        pydict.set_item("st_atime", self.0.st_atime)?;
        pydict.set_item("st_size", self.0.st_size)?;
        pydict.set_item("st_blksize", self.0.st_blksize)?;
        pydict.set_item("st_blocks", self.0.st_blocks)?;
        pydict.set_item("st_mode", self.0.st_mode)?;
        pydict.set_item("st_nlink", self.0.st_nlink)?;
        pydict.set_item("st_uid", self.0.st_uid)?;
        pydict.set_item("st_gid", self.0.st_gid)?;
        pydict.set_item("st_ino", self.0.st_ino)?;
        pydict.set_item("st_dev", self.0.st_dev)?;
        pydict.set_item("st_rdev", self.0.st_rdev)?;
        Ok(pydict.into_any().unbind())
    }

    #[cfg(feature = "speedy")]
    fn to_speedy(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.0.write_to_vec() {
            Ok(v) => Ok(PyBytes::new_with(py, v.len(), |b| {
                b.copy_from_slice(&v);
                Ok(())
            })?
            .into()),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "bincode")]
    fn to_bincode(&self, py: Python) -> PyResult<Py<PyBytes>> {
        match self.0.to_vec() {
            Ok(v) => Ok(PyBytes::new_with(py, v.len(), |b| {
                b.copy_from_slice(&v);
                Ok(())
            })?
            .into()),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    #[cfg(feature = "json")]
    fn to_json(&self) -> PyResult<String> {
        self.0
            .to_json()
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }
}
