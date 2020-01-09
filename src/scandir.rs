use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{UNIX_EPOCH, Instant};
use std::thread;
use std::sync::{Arc, Mutex, Weak};

use crossbeam_channel as channel;
#[cfg(unix)]
use expanduser::expanduser;
use jwalk::WalkDir;

use pyo3::exceptions::{self, ValueError};
use pyo3::prelude::*;
use pyo3::types::{PyType, PyAny, PyTuple, PyDict};
use pyo3::{Python, wrap_pyfunction, PyContextProtocol, PyIterProtocol};

use crate::def::*;

#[derive(Debug, Clone)]
pub enum Stats {
    DirEntry(DirEntry),
    Error(String),
    Duration(f64),
}

#[derive(Debug, Clone)]
pub struct Entry {
    path: String,
    entry: Stats,
}

impl ToPyObject for Entry {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        match &self.entry {
            Stats::DirEntry(e) => PyTuple::new(py, &[self.path.to_object(py), PyRef::new(py, e.clone()).unwrap().to_object(py)]).into(),
            Stats::Error(e) => PyTuple::new(py, &[self.path.to_object(py), e.to_object(py)]).into(),
            Stats::Duration(e) => PyTuple::new(py, &[self.path.to_object(py), e.to_object(py)]).into(),
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Entries {
    pub entries: Vec<Entry>,
    pub duration: f64,
}

#[pymethods]
impl Entries {
    #[getter]
    fn entries(&self) -> PyResult<PyObject> {
        Ok(PyTuple::new(GILGuard::acquire().python(), &self.entries).into())
    }

    #[getter]
    fn duration(&self) -> PyResult<f64> {
       Ok(self.duration)
    }
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Entries {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

fn create_entry(entry: &std::result::Result<jwalk::core::dir_entry::DirEntry<()>, std::io::Error>) -> Entry {
    match entry {
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
            let mut uid: u32 = 0;
            let mut gid: u32 = 0;
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
                uid: uid,
                gid: gid,
                rdev: rdev,
            };
            Entry{
                path: key.to_str().unwrap().to_string(),
                entry: Stats::DirEntry(entry),
            }
        },
        // TODO: Need to fetch failed path from somewhere
        Err(e) => Entry {
            path: String::from("?"),
            entry: Stats::Error(e.to_string()),
        }
    }
}

fn rs_entries(
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    metadata: bool,
    metadata_ext: bool,
    max_depth: usize,
    result: Arc<Mutex<Entries>>,
    alive: Option<Arc<AtomicBool>>,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    let start_time = Instant::now();
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .preload_metadata(metadata)
        .preload_metadata_ext(metadata_ext)
        .max_depth(max_depth)
    {
        let entry = create_entry(&entry);
        let mut result_locked = result.lock().unwrap();
        result_locked.entries.push(entry);
        result_locked.duration = start_time.elapsed().as_millis() as f64 * 0.001;
        match &alive {
            Some(a) => if !a.load(Ordering::Relaxed) {
                break;
            },
            None => {},
        }
    }
    result.lock().unwrap().duration = start_time.elapsed().as_millis() as f64 * 0.001;
}

fn rs_entries_iter(
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    metadata: bool,
    metadata_ext: bool,
    max_depth: usize,
    alive: Option<Arc<AtomicBool>>,
    tx: Option<channel::Sender<Entry>>,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    let start_time = Instant::now();
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .preload_metadata(metadata)
        .preload_metadata_ext(metadata_ext)
        .max_depth(max_depth)
    {
        let entry = create_entry(&entry);
        match &tx {
            Some(tx) => {
                if tx.send(entry).is_err() {
                    return;
                }
            },
            None => {},
        }
        match &alive {
            Some(a) => if !a.load(Ordering::Relaxed) {
                break;
            },
            None => {},
        }
    }
    match &tx {
        Some(tx) => {
            let _ = tx.send(Entry {
                path: String::from("?"),
                entry: Stats::Duration(start_time.elapsed().as_millis() as f64 * 0.001),
            });
        },
        None => {}
    }
}

#[pyfunction]
pub fn entries(
    py: Python,
    root_path: String,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
    max_depth: Option<usize>,
) -> PyResult<Entries> {
    let result = Arc::new(Mutex::new(Entries {
        entries: Vec::new(),
        duration: 0.0,
    }));
    let result_cloned = result.clone();
    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        rs_entries(
            root_path,
            sorted.unwrap_or(false),
            skip_hidden.unwrap_or(false),
            metadata.unwrap_or(false),
            metadata_ext.unwrap_or(false),
            max_depth.unwrap_or(::std::usize::MAX),
            result_cloned,
            None
        );
        Ok(())
    });
    match rc {
        Err(e) => return Err(exceptions::RuntimeError::py_err(e.to_string())),
        _ => (),
    }
    let result_cloned = result.lock().unwrap().clone();
    Ok(result_cloned.into())
}

#[pyclass]
#[derive(Debug)]
pub struct Scandir {
    // Options
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    metadata: bool,
    metadata_ext: bool,
    max_depth: usize,
    // Results
    entries: Arc<Mutex<Entries>>,
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

    fn rs_start(&mut self,
        tx: Option<channel::Sender<Entry>>,
    ) -> bool {
        if self.thr.is_some() {
            return false;
        }
        if self.has_results {
            self.rs_init();
        }
        let root_path = String::from(&self.root_path);
        let sorted = self.sorted;
        let skip_hidden = self.skip_hidden;
        let metadata = self.metadata;
        let metadata_ext = self.metadata_ext;
        let max_depth = self.max_depth;
        let entries = self.entries.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            if tx.is_none() {
                rs_entries(
                    root_path,
                    sorted,
                    skip_hidden,
                    metadata,
                    metadata_ext,
                    max_depth,
                    entries,
                    Some(alive),
                )
            } else {
                rs_entries_iter(
                    root_path,
                    sorted,
                    skip_hidden,
                    metadata,
                    metadata_ext,
                    max_depth,
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
    fn __new__(
        obj: &PyRawObject,
        root_path: &str,
        sorted: Option<bool>,
        skip_hidden: Option<bool>,
        metadata: Option<bool>,
        metadata_ext: Option<bool>,
        max_depth: Option<usize>,
    ) {
        obj.init(Scandir {
            root_path: String::from(root_path),
            sorted: sorted.unwrap_or(false),
            skip_hidden: skip_hidden.unwrap_or(false),
            metadata: metadata.unwrap_or(false),
            metadata_ext: metadata_ext.unwrap_or(false),
            max_depth: max_depth.unwrap_or(::std::usize::MAX),
            entries: Arc::new(Mutex::new(Entries {
                entries: Vec::new(),
                duration: 0.0,
            })),
            thr: None,
            alive: None,
            rx: None,
            has_results: false,
        });
    }

    #[getter]
    fn entries(&self) -> PyResult<Entries> {
        Ok(Arc::clone(&self.entries).lock().unwrap().clone())
    }

    fn has_results(&self) -> PyResult<bool> {
        Ok(self.has_results)
    }

    fn as_dict(&self) -> PyResult<PyObject> {
        let entries_locked = self.entries.lock().unwrap();
        let gil = GILGuard::acquire();
        let py = gil.python();
        let pyresult = PyDict::new(py);
        for entry in &entries_locked.entries {
            match &entry.entry {
                Stats::DirEntry(e) => pyresult.set_item(entry.path.to_object(py), PyRef::new(py, e.clone()).unwrap()).unwrap(),
                Stats::Error(e) => pyresult.set_item(entry.path.to_object(py), e).unwrap(),
                Stats::Duration(_) => {},
            }
        }
        Ok(pyresult.into())
    }

    fn collect(&self) -> PyResult<Entries> {
        rs_entries(
            self.root_path.clone(),
            self.sorted,
            self.skip_hidden,
            self.metadata,
            self.metadata_ext,
            self.max_depth,
            self.entries.clone(),
            None
        );
        Ok(Arc::clone(&self.entries).lock().unwrap().clone())
    }

    fn start(&mut self) -> PyResult<bool> {
        if !self.rs_start(None) {
            return Err(exceptions::RuntimeError::py_err("Thread already running"));
        }
        Ok(true)
    }

    fn stop(&mut self) -> PyResult<bool> {
        if !self.rs_stop() {
            return Err(exceptions::RuntimeError::py_err("Thread not running"));
        }
        Ok(true)
    }

    fn busy(&self) -> PyResult<bool> {
        match &self.alive {
            Some(alive) => match alive.upgrade() {
                Some(_) => Ok(true),
                None => return Ok(false),
            },
            None => Ok(false),
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
impl<'p> PyContextProtocol<'p> for Scandir {
    fn __enter__(&'p mut self) -> PyResult<()> {
        if !self.rs_start(None) {
            return Err(exceptions::RuntimeError::py_err("Thread already running"));
        }
        Ok(())
    }

    fn __exit__(
        &mut self,
        ty: Option<&'p PyType>,
        _value: Option<&'p PyAny>,
        _traceback: Option<&'p PyAny>,
    ) -> PyResult<bool> {
        if !self.rs_stop() {
            return Ok(false);
        }
        if ty == Some(GILGuard::acquire().python().get_type::<ValueError>()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[pyproto]
impl<'p> PyIterProtocol for Scandir {
    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<Scandir>> {
        if slf.thr.is_some() {
            return Err(exceptions::RuntimeError::py_err("Thread already running"));
        }
        let (tx, rx) = channel::unbounded();
        slf.rx = Some(rx);
        slf.rs_start(Some(tx));
        Ok(slf.into())
    }

    fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        let gil = GILGuard::acquire();
        let py = gil.python();
        match &slf.rx {
            Some(rx) => match rx.recv() {
                Ok(val) => match &val.entry {
                    Stats::DirEntry(_) => Ok(Some(val.to_object(py))),
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

#[pymodule(scandir)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Scandir>()?;
    m.add_wrapped(wrap_pyfunction!(entries))?;
    Ok(())
}
