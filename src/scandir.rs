use std::fs;
use std::fs::Metadata;
use std::io::{Error, ErrorKind};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Instant, UNIX_EPOCH};

use crossbeam_channel as channel;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyString, PyTuple, PyType};
use pyo3::{wrap_pyfunction, PyIterProtocol, Python};

use jwalk::WalkDirGeneric;

use crate::common::check_and_expand_path;
use crate::common::{create_filter, filter_children};
use crate::cst::*;
use crate::def::*;

static BUSY: AtomicBool = AtomicBool::new(false);
static COUNT: AtomicU32 = AtomicU32::new(0);

#[pyfunction]
pub fn ts_busy() -> bool {
    BUSY.load(Ordering::Relaxed)
}

#[pyfunction]
pub fn ts_count() -> u32 {
    COUNT.load(Ordering::Relaxed)
}

#[derive(Debug, Clone)]
pub enum Stats {
    ScandirResult(ScandirResult),
    Error(String),
    Duration(f64),
}

/// Scandir result
#[derive(Debug, Clone)]
pub struct Entry {
    /// Absolute path
    pub path: String,
    /// File stats
    pub entry: Stats,
}

impl ToPyObject for Entry {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        match &self.entry {
            Stats::ScandirResult(e) => PyTuple::new(
                py,
                &[
                    self.path.to_object(py),
                    match e {
                        ScandirResult::DirEntry(e) => {
                            PyCell::new(py, e.clone()).unwrap().to_object(py)
                        }
                        ScandirResult::DirEntryExt(e) => {
                            PyCell::new(py, e.clone()).unwrap().to_object(py)
                        }
                        ScandirResult::DirEntryFull(e) => {
                            PyCell::new(py, e.clone()).unwrap().to_object(py)
                        }
                        ScandirResult::Error(e) => PyString::new(py, &e).to_object(py),
                    },
                ],
            )
            .into(),
            Stats::Error(e) => PyTuple::new(py, &[self.path.to_object(py), e.to_object(py)]).into(),
            Stats::Duration(e) => {
                PyTuple::new(py, &[self.path.to_object(py), e.to_object(py)]).into()
            }
        }
    }
}

/// Scandir results
#[pyclass]
#[derive(Debug, Clone)]
pub struct Entries {
    /// List of scandir results
    pub entries: Vec<Entry>,
    /// Time used for iteration
    #[pyo3(get)]
    pub duration: f64,
}

#[pymethods]
impl Entries {
    #[getter]
    fn entries(&self) -> PyObject {
        PyTuple::new(Python::acquire_gil().python(), &self.entries).into()
    }
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Entries {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyproto]
impl pyo3::class::PySequenceProtocol for Entries {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.entries.len())
    }
}

fn create_entry(
    root_path_len: usize,
    return_type: u8,
    dir_entry: &jwalk::DirEntry<((), Option<Result<Metadata, Error>>)>,
) -> (bool, Entry) {
    let file_type = dir_entry.file_type;
    let mut st_ctime: f64 = 0.0;
    let mut st_mtime: f64 = 0.0;
    let mut st_atime: f64 = 0.0;
    let mut st_mode: u32 = 0;
    let mut st_ino: u64 = 0;
    let mut st_dev: u64 = 0;
    let mut st_nlink: u64 = 0;
    let mut st_size: u64 = 0;
    let mut st_blksize: u64 = 4096;
    let mut st_blocks: u64 = 0;
    #[cfg(unix)]
    let mut st_uid: u32 = 0;
    #[cfg(windows)]
    let st_uid: u32 = 0;
    #[cfg(unix)]
    let mut st_gid: u32 = 0;
    #[cfg(windows)]
    let st_gid: u32 = 0;
    #[cfg(unix)]
    let mut st_rdev: u64 = 0;
    #[cfg(windows)]
    let st_rdev: u64 = 0;
    if let Ok(metadata) = fs::metadata(dir_entry.path()) {
        let duration = metadata
            .created()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap();
        st_ctime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let duration = metadata
            .modified()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap();
        st_mtime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let duration = metadata
            .accessed()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap();
        st_atime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        if return_type > RETURN_TYPE_BASE {
            #[cfg(unix)]
            {
                st_mode = metadata.mode();
                st_ino = metadata.ino();
                st_dev = metadata.dev() as u64;
                st_nlink = metadata.nlink() as u64;
                st_size = metadata.size();
                st_blksize = metadata.blksize();
                st_blocks = metadata.blocks();
                st_uid = metadata.uid();
                st_gid = metadata.gid();
                st_rdev = metadata.rdev();
            }
            #[cfg(windows)]
            {
                st_mode = metadata.file_attributes();
                if let Some(ino) = metadata.file_index() {
                    st_ino = ino;
                }
                if let Some(dev) = metadata.volume_serial_number() {
                    st_dev = dev as u64;
                }
                if let Some(nlink) = metadata.number_of_links() {
                    st_nlink = nlink as u64;
                }
                st_size = metadata.file_size();
                st_blksize = 4096;
                st_blocks = st_size >> 12;
                if st_blocks << 12 < st_size {
                    st_blocks += 1;
                }
            }
        }
    }
    let mut key = dir_entry.parent_path.to_path_buf();
    let file_name = dir_entry.file_name.clone().into_string().unwrap();
    key.push(&file_name);
    let key = key.to_str().unwrap().to_string();
    let is_file = file_type.is_file();
    let entry: ScandirResult = match return_type {
        RETURN_TYPE_FAST | RETURN_TYPE_BASE => ScandirResult::DirEntry(DirEntry {
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
            st_ctime: st_ctime,
            st_mtime: st_mtime,
            st_atime: st_atime,
        }),
        RETURN_TYPE_EXT => ScandirResult::DirEntryExt(DirEntryExt {
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
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
        }),
        RETURN_TYPE_FULL => ScandirResult::DirEntryFull(DirEntryFull {
            name: file_name,
            path: key.get(root_path_len..).unwrap_or("").to_string(),
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
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
        }),
        _ => ScandirResult::Error("Wrong return type!".to_string()),
    };
    (
        is_file,
        Entry {
            path: key,
            entry: Stats::ScandirResult(entry),
        },
    )
}

fn rs_entries(
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    filter: Option<Filter>,
    return_type: u8,
    result: Arc<Mutex<Entries>>,
    alive: Option<Arc<AtomicBool>>,
) {
    BUSY.store(true, Ordering::Relaxed);
    let start_time = Instant::now();
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    };
    let root_path_len = root_path.to_string_lossy().len() + 1;
    let result_clone = result.clone();
    for _ in WalkDirGeneric::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
        .process_read_dir(move |_, _, _, children| {
            filter_children(children, &filter, root_path_len);
            children.iter_mut().for_each(|dir_entry_result| {
                match &alive {
                    Some(a) => {
                        if !a.load(Ordering::Relaxed) {
                            BUSY.store(false, Ordering::Relaxed);
                            return;
                        }
                    }
                    None => {}
                }
                if let Ok(dir_entry) = dir_entry_result {
                    let (is_file, entry) = create_entry(root_path_len, return_type, dir_entry);
                    let mut result_locked = result_clone.lock().unwrap();
                    result_locked.entries.push(entry);
                    if is_file {
                        COUNT.store(COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                    }
                }
            });
        })
    {}
    result.lock().unwrap().duration = start_time.elapsed().as_millis() as f64 * 0.001;
    BUSY.store(false, Ordering::Relaxed);
}

fn rs_entries_iter(
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    filter: Option<Filter>,
    return_type: u8,
    alive: Option<Arc<AtomicBool>>,
    tx: Option<channel::Sender<Entry>>,
) {
    BUSY.store(true, Ordering::Relaxed);
    let start_time = Instant::now();
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    }
    let root_path_len = root_path.to_string_lossy().len() + 1;
    let tx_clone = tx.clone();
    for _ in WalkDirGeneric::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
        .process_read_dir(move |_, _, _, children| {
            filter_children(children, &filter, root_path_len);
            children.iter_mut().for_each(|dir_entry_result| {
                match &alive {
                    Some(a) => {
                        if !a.load(Ordering::Relaxed) {
                            BUSY.store(false, Ordering::Relaxed);
                            return;
                        }
                    }
                    None => {}
                }
                if let Ok(dir_entry) = dir_entry_result {
                    let (is_file, entry) = create_entry(root_path_len, return_type, dir_entry);
                    match &tx_clone {
                        Some(tx_clone) => {
                            if tx_clone.send(entry).is_err() {
                                BUSY.store(false, Ordering::Relaxed);
                                return;
                            }
                        }
                        None => {}
                    }
                    if is_file {
                        COUNT.store(COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                    }
                }
            });
        })
    {}
    match &tx {
        Some(tx) => {
            let _ = tx.send(Entry {
                path: String::from("?"),
                entry: Stats::Duration(start_time.elapsed().as_millis() as f64 * 0.001),
            });
        }
        None => {}
    }
    BUSY.store(false, Ordering::Relaxed);
}

#[pyfunction]
pub fn entries<'a>(
    py: Python,
    root_path: String,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
    max_depth: Option<usize>,
    dir_include: Option<Vec<String>>,
    dir_exclude: Option<Vec<String>>,
    file_include: Option<Vec<String>>,
    file_exclude: Option<Vec<String>>,
    case_sensitive: Option<bool>,
    return_type: Option<u8>,
) -> PyResult<Entries> {
    let return_type = return_type.unwrap_or(RETURN_TYPE_BASE);
    if return_type > RETURN_TYPE_FULL {
        return Err(PyValueError::new_err("Invalid return type".to_string()));
    }
    let root_path = match check_and_expand_path(&root_path) {
        Ok(p) => p,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Err(PyFileNotFoundError::new_err(e.to_string())),
            _ => return Err(PyException::new_err(e.to_string())),
        },
    };
    let filter = match create_filter(
        dir_include,
        dir_exclude,
        file_include,
        file_exclude,
        case_sensitive,
    ) {
        Ok(f) => f,
        Err(e) => return Err(PyValueError::new_err(e.to_string())),
    };
    let result = Arc::new(Mutex::new(Entries {
        entries: Vec::new(),
        duration: 0.0,
    }));
    let result_cloned = result.clone();
    let rc: Result<(), Error> = py.allow_threads(|| {
        rs_entries(
            root_path,
            sorted.unwrap_or(false),
            skip_hidden.unwrap_or(false),
            max_depth.unwrap_or(::std::usize::MAX),
            filter,
            return_type,
            result_cloned,
            None,
        );
        Ok(())
    });
    match rc {
        Err(e) => return Err(PyRuntimeError::new_err(e.to_string())),
        _ => (),
    }
    let result_cloned = result.lock().unwrap().clone();
    Ok(result_cloned.into())
}

/// Class for iterating a file tree and returning `Entry` objects
#[pyclass]
#[derive(Debug)]
pub struct Scandir {
    // Options
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    max_depth: usize,
    return_type: u8,
    filter: Option<Filter>,
    // Results
    pub entries: Arc<Mutex<Entries>>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Option<Weak<AtomicBool>>,
    rx: Option<channel::Receiver<Entry>>,
    has_results: bool,
}

impl Scandir {
    fn rs_init(&self) {
        let mut entries_locked = self.entries.lock().unwrap();
        entries_locked.entries.clear();
    }

    fn rs_start(&mut self, tx: Option<channel::Sender<Entry>>) -> bool {
        if self.thr.is_some() {
            return false;
        }
        if self.has_results {
            self.rs_init();
        }
        let root_path = self.root_path.clone();
        let sorted = self.sorted;
        let skip_hidden = self.skip_hidden;
        let max_depth = self.max_depth;
        let filter = self.filter.clone();
        let return_type = self.return_type;
        let entries = self.entries.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            if tx.is_none() {
                rs_entries(
                    root_path,
                    sorted,
                    skip_hidden,
                    max_depth,
                    filter,
                    return_type,
                    entries,
                    Some(alive),
                )
            } else {
                rs_entries_iter(
                    root_path,
                    sorted,
                    skip_hidden,
                    max_depth,
                    filter,
                    return_type,
                    Some(alive),
                    tx,
                )
            }
        }));
        true
    }

    fn rs_stop(&mut self) -> bool {
        match &self.alive {
            Some(alive) => match alive.upgrade() {
                Some(alive) => (*alive).store(false, Ordering::Relaxed),
                None => return false,
            },
            None => {}
        }
        self.thr.take().map(thread::JoinHandle::join);
        self.has_results = true;
        true
    }
}

#[pymethods]
impl Scandir {
    #[new]
    pub fn new(
        root_path: &str,
        sorted: Option<bool>,
        skip_hidden: Option<bool>,
        max_depth: Option<usize>,
        dir_include: Option<Vec<String>>,
        dir_exclude: Option<Vec<String>>,
        file_include: Option<Vec<String>>,
        file_exclude: Option<Vec<String>>,
        case_sensitive: Option<bool>,
        return_type: Option<u8>,
    ) -> PyResult<Self> {
        let return_type = return_type.unwrap_or(RETURN_TYPE_BASE);
        if return_type > RETURN_TYPE_FULL {
            return Err(PyValueError::new_err(
                "Parameter return_type has invalid value",
            ));
        }
        let root_path = match check_and_expand_path(&root_path) {
            Ok(p) => p,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => return Err(PyFileNotFoundError::new_err(e.to_string())),
                _ => return Err(PyException::new_err(e.to_string())),
            },
        };
        let filter = match create_filter(
            dir_include,
            dir_exclude,
            file_include,
            file_exclude,
            case_sensitive,
        ) {
            Ok(f) => f,
            Err(e) => {
                return Err(PyValueError::new_err(e.to_string()));
            }
        };
        Ok(Scandir {
            root_path: root_path,
            sorted: sorted.unwrap_or(false),
            skip_hidden: skip_hidden.unwrap_or(false),
            max_depth: max_depth.unwrap_or(::std::usize::MAX),
            return_type: return_type,
            filter: filter,
            entries: Arc::new(Mutex::new(Entries {
                entries: Vec::new(),
                duration: 0.0,
            })),
            thr: None,
            alive: None,
            rx: None,
            has_results: false,
        })
    }

    fn __enter__(&mut self) -> PyResult<()> {
        if !self.rs_start(None) {
            return Err(PyRuntimeError::new_err("Thread already running"));
        }
        Ok(())
    }

    fn __exit__(
        &mut self,
        ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        if !self.rs_stop() {
            return Ok(false);
        }
        if ty == Some(Python::acquire_gil().python().get_type::<PyValueError>()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[getter]
    pub fn entries(&self) -> Entries {
        Arc::clone(&self.entries).lock().unwrap().clone()
    }

    pub fn has_results(&self) -> bool {
        self.has_results
    }

    pub fn as_dict(&self) -> PyObject {
        let entries_locked = self.entries.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyresult = PyDict::new(py);
        for entry in &entries_locked.entries {
            match &entry.entry {
                Stats::ScandirResult(e) => pyresult
                    .set_item(
                        entry.path.to_object(py),
                        match e {
                            ScandirResult::DirEntry(e) => {
                                PyCell::new(py, e.clone()).unwrap().to_object(py)
                            }
                            ScandirResult::DirEntryExt(e) => {
                                PyCell::new(py, e.clone()).unwrap().to_object(py)
                            }
                            ScandirResult::DirEntryFull(e) => {
                                PyCell::new(py, e.clone()).unwrap().to_object(py)
                            }
                            ScandirResult::Error(e) => PyString::new(py, &e).to_object(py),
                        },
                    )
                    .unwrap(),
                Stats::Error(e) => pyresult.set_item(entry.path.to_object(py), e).unwrap(),
                Stats::Duration(_) => {}
            }
        }
        pyresult.into()
    }

    pub fn collect(&mut self) -> PyResult<Entries> {
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        let gil = Python::acquire_gil();
        let rc: Result<(), Error> = gil.python().allow_threads(|| {
            rs_entries(
                self.root_path.clone(),
                self.sorted,
                self.skip_hidden,
                self.max_depth,
                self.filter.clone(),
                self.return_type,
                self.entries.clone(),
                None,
            );
            Ok(())
        });
        self.alive = None;
        match rc {
            Err(e) => return Err(PyRuntimeError::new_err(e.to_string())),
            _ => (),
        }
        self.has_results = true;
        Ok(Arc::clone(&self.entries).lock().unwrap().clone())
    }

    pub fn start(&mut self) -> PyResult<bool> {
        if !self.rs_start(None) {
            return Err(PyRuntimeError::new_err("Thread already running"));
        }
        Ok(true)
    }

    pub fn stop(&mut self) -> PyResult<bool> {
        if !self.rs_stop() {
            return Err(PyRuntimeError::new_err("Thread not running"));
        }
        Ok(true)
    }

    pub fn busy(&self) -> bool {
        match &self.alive {
            Some(alive) => match alive.upgrade() {
                Some(_) => true,
                None => return false,
            },
            None => false,
        }
    }
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Scandir {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyproto]
impl<'p> PyIterProtocol for Scandir {
    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<Scandir>> {
        if slf.thr.is_some() {
            return Err(PyRuntimeError::new_err("Thread already running"));
        }
        let (tx, rx) = channel::unbounded();
        slf.rx = Some(rx);
        slf.rs_start(Some(tx));
        Ok(slf.into())
    }

    fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match &slf.rx {
            Some(rx) => match rx.recv() {
                Ok(val) => match &val.entry {
                    Stats::ScandirResult(_) => Ok(Some(val.to_object(py))),
                    Stats::Error(_) => Ok(Some(val.to_object(py))),
                    Stats::Duration(d) => {
                        let mut entries_locked = slf.entries.lock().unwrap();
                        entries_locked.duration = *d;
                        Ok(None)
                    }
                },
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }
}

#[pymodule]
#[pyo3(name = "scandir")]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Scandir>()?;
    m.add_wrapped(wrap_pyfunction!(entries))?;
    m.add_wrapped(wrap_pyfunction!(ts_busy))?;
    m.add_wrapped(wrap_pyfunction!(ts_count))?;
    Ok(())
}
