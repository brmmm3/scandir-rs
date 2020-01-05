use std::time::UNIX_EPOCH;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::thread;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use jwalk::WalkDir;
#[cfg(unix)]
use expanduser::expanduser;

use pyo3::prelude::*;
use pyo3::types::{PyType, PyAny, PyDict};
use pyo3::{Python, wrap_pyfunction, PyContextProtocol, PyIterProtocol};
use pyo3::exceptions::{self, ValueError};

use crate::def::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Entries {
    pub entries: HashMap<String, DirEntry>,
    pub errors: Vec<String>,
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Entries {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
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
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .preload_metadata(metadata)
        .preload_metadata_ext(metadata_ext)
        .max_depth(max_depth)
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
                let mut size: u64 = 0;
                let mut blksize: u64 = 4096;
                let mut blocks: u64 = 0;
                #[cfg(unix)]
                let mut uid: u32 = 0;
                #[cfg(unix)]
                let mut gid: u32 = 0;
                #[cfg(unix)]
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
                        #[cfg(unix)]
                        uid: uid,
                        #[cfg(unix)]
                        gid: gid,
                        #[cfg(unix)]
                        rdev: rdev,
                    };
                let mut result_locked = result.lock().unwrap();
                result_locked.entries.insert(key.to_str().unwrap().to_string(), entry);
            }
            Err(e) => result.lock().unwrap().errors.push(e.to_string())  // TODO: Need to fetch failed path from somewhere
        };
        match &alive {
            Some(a) => if !a.load(Ordering::Relaxed) {
                break;
            },
            None => {},
        }
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
        entries: HashMap::new(),
        errors: Vec::new(),
    }));
    let result_cloned = result.clone();
    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        rs_entries(root_path,
                   sorted.unwrap_or(false),
                   skip_hidden.unwrap_or(false),
                   metadata.unwrap_or(false),
                   metadata_ext.unwrap_or(false),
                   max_depth.unwrap_or(::std::usize::MAX),
                   result_cloned, None);
        Ok(())
    });
    match rc {
        Err(e) => return Err(exceptions::RuntimeError::py_err(e.to_string())),
        _ => ()
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
    has_results: bool,
    start_time: Instant,
    duration: Duration,
}

impl Scandir {
    fn rs_init(&self) {
        let mut entries_locked = self.entries.lock().unwrap();
        entries_locked.entries.clear();
        entries_locked.errors.clear();
    }

    fn rs_start(&mut self) -> bool {
        if self.thr.is_some() {
            return false
        }
        self.start_time = Instant::now();
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
            rs_entries(root_path,
                       sorted, skip_hidden, metadata, metadata_ext, max_depth,
                       entries, Some(alive))
        }));
        true
    }

    fn rs_stop(&mut self) -> bool {
        match &self.alive {
            Some(alive) => match alive.upgrade() {
                Some(alive) => (*alive).store(false, Ordering::Relaxed),
                None => return false,
            },
            None => {},
        }
        self.thr.take().map(thread::JoinHandle::join);
        self.duration = self.start_time.elapsed();
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
                entries: HashMap::new(),
                errors: Vec::new(),
            })),
            thr: None,
            alive: None,
            has_results: false,
            start_time: Instant::now(),
            duration: Duration::new(0, 0),
        });
    }

    #[getter]
    fn entries(&self) -> PyResult<Entries> {
       Ok(Arc::clone(&self.entries).lock().unwrap().clone())
    }

    #[getter]
    fn has_results(&self) -> PyResult<bool> {
       Ok(self.has_results)
    }

    #[getter]
    fn duration(&self) -> PyResult<f64> {
       Ok(self.duration.as_secs() as f64 + self.duration.subsec_nanos() as f64 * 1e-9)
    }

    fn as_dict(&self) -> PyResult<PyObject> {
        let gil = GILGuard::acquire();
        let py = gil.python();
        let mut entries_locked = self.entries.lock().unwrap();
        let pyresult = PyDict::new(gil.python());
        for (k, v) in entries_locked.entries.drain() {
            pyresult.set_item(k, PyRef::new(py, v).unwrap())?;
        }
        if !entries_locked.errors.is_empty() {
            pyresult.set_item(py.None(), entries_locked.errors.to_vec())?;
        }
        Ok(pyresult.into())
    }

    fn collect(&self) -> PyResult<Entries> {
        rs_entries(self.root_path.clone(),
                   self.sorted, self.skip_hidden, self.metadata, self.metadata_ext, self.max_depth,
                   self.entries.clone(), None);
        Ok(Arc::clone(&self.entries).lock().unwrap().clone())
    }

    fn start(&mut self) -> PyResult<bool> {
        if !self.rs_start() {
            return Err(exceptions::RuntimeError::py_err("Thread already running"))
        }
        Ok(true)
    }

    fn stop(&mut self) -> PyResult<bool> {
        if !self.rs_stop() {
            return Err(exceptions::RuntimeError::py_err("Thread not running"))
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
        self.rs_start();
        Ok(())
    }

    fn __exit__(
        &mut self,
        ty: Option<&'p PyType>,
        _value: Option<&'p PyAny>,
        _traceback: Option<&'p PyAny>,
    ) -> PyResult<bool> {
        if !self.rs_stop() {
            return Ok(false)
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
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Scandir>> {
        Ok(slf.into())
    }

    fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<Entries>> {
        let entries_locked = slf.entries.lock().unwrap();
        if entries_locked.entries.is_empty()
                && entries_locked.errors.is_empty() {
            return Ok(None)
        }
        Ok(Some(entries_locked.clone()))
    }
}

#[pymodule(scandir)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Scandir>()?;
    m.add_wrapped(wrap_pyfunction!(entries))?;
    Ok(())
}
