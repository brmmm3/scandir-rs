use std::thread;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use jwalk::WalkDir;
#[cfg(unix)]
use expanduser::expanduser;

use pyo3::prelude::*;
use pyo3::types::{PyType, PyAny, PyDict};
use pyo3::{Python, wrap_pyfunction, PyContextProtocol};
use pyo3::exceptions::{self, ValueError};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Statistics {
    pub dirs: u32,
    pub files: u32,
    pub slinks: u32,
    pub hlinks: u32,
    pub devices: u32,
    pub pipes: u32,
    pub size: u64,
    pub usage: u64,
    pub errors: Vec<String>,
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Statistics {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

fn counter(
    root_path: String,
    skip_hidden: bool,
    metadata: bool,
    metadata_ext: bool,
    max_depth: usize,
    statistics: Arc<Mutex<Statistics>>,
    alive: Option<Arc<AtomicBool>>,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    let mut dirs: u32 = 0;
    let mut files: u32 = 0;
    let mut slinks: u32 = 0;
    let mut hlinks: u32 = 0;
    let mut size: u64 = 0;
    let mut usage: u64 = 0;
    let mut errors: Vec<String> = Vec::new();
    #[cfg(unix)]
    let mut devices: u32 = 0;
    #[cfg(unix)]
    let mut pipes: u32 = 0;
    let mut cnt: u32 = 0;
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(false)
        .preload_metadata(metadata)
        .preload_metadata_ext(metadata_ext)
        .max_depth(max_depth)
    {
        match &entry {
            Ok(v) => {
                let file_type = v.file_type_result.as_ref().unwrap();
                if file_type.is_dir() {
                    dirs += 1;
                }
                else if file_type.is_file() {
                    files += 1;
                }
                else if file_type.is_symlink() {
                    slinks += 1;
                }
                if v.metadata_result.is_some() {
                    let metadata_ext = v.ext.as_ref();
                    if metadata_ext.is_some() {
                        let metadata_ext = metadata_ext.unwrap().as_ref().unwrap();
                        if metadata_ext.nlink > 1 {
                            hlinks += 1;
                        }
                        size += metadata_ext.size;
                        #[cfg(unix)]
                        {
                            if metadata_ext.rdev > 0 {
                                devices += 1;
                            }
                            if (metadata_ext.mode & 4096) != 0 {
                                pipes += 1;
                            }
                            usage += metadata_ext.blocks * 512;
                        }
                        #[cfg(windows)]
                        {
                            let mut blocks = metadata_ext.size >> 12;
                            if blocks << 12 < metadata_ext.size {
                                blocks += 1;
                            }
                            usage += blocks << 12;
                        }
                    }
                }
            }
            Err(e) => errors.push(e.to_string())  // TODO: Need to fetch failed path from somewhere
        };
        cnt += 1;
        if cnt >= 1000 {
            let mut stats = statistics.lock().unwrap();
            stats.dirs = dirs;
            stats.files = files;
            stats.slinks = slinks;
            stats.hlinks = hlinks;
            stats.size = size;
            stats.usage = usage;
            if stats.errors.len() < errors.len() {
                stats.errors.extend_from_slice(&errors);
                errors.clear();
            }
            #[cfg(unix)]
            {
                stats.devices = devices;
                stats.pipes = pipes;
            }
            cnt = 0;
        }
        match &alive {
            Some(a) => if !a.load(Ordering::Relaxed) {
                break;
            },
            None => {},
        }
    }
    {
        let mut stats = statistics.lock().unwrap();
        stats.dirs = dirs;
        stats.files = files;
        stats.slinks = slinks;
        stats.hlinks = hlinks;
        stats.size = size;
        stats.usage = usage;
        if stats.errors.len() < errors.len() {
            stats.errors.extend_from_slice(&errors);
            errors.clear();
        }
        #[cfg(unix)]
        {
            stats.devices = devices;
            stats.pipes = pipes;
        }
    }
}

#[pyfunction]
pub fn count(
    py: Python,
    root_path: String,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
    max_depth: Option<usize>,
) -> PyResult<PyObject> {
    let statistics = Arc::new(Mutex::new(Statistics { 
        dirs: 0,
        files: 0,
        slinks: 0,
        hlinks: 0,
        devices: 0,
        pipes: 0,
        size: 0,
        usage: 0,
        errors: Vec::new(),
    }));
    let stats = statistics.clone();
    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        counter(root_path,
                skip_hidden.unwrap_or(false),
                metadata.unwrap_or(false),
                metadata_ext.unwrap_or(false),
                max_depth.unwrap_or(::std::usize::MAX),
                stats, None);
        Ok(())
    });
    let pyresult = PyDict::new(py);
    match rc {
        Err(e) => { pyresult.set_item("error", e.to_string()).unwrap();
                    return Ok(pyresult.into())
                  },
        _ => ()
    }
    {
        let stats = statistics.lock().unwrap();
        if stats.dirs > 0 {
            pyresult.set_item("dirs", stats.dirs).unwrap();
        }
        if stats.files > 0 {
            pyresult.set_item("files", stats.files).unwrap();
        }
        if stats.slinks > 0 {
            pyresult.set_item("slinks", stats.slinks).unwrap();
        }
        if stats.hlinks > 0 {
            pyresult.set_item("hlinks", stats.hlinks).unwrap();
        }
        if stats.devices > 0 {
            pyresult.set_item("devices", stats.devices).unwrap();
        }
        if stats.pipes > 0 {
            pyresult.set_item("pipes", stats.pipes).unwrap();
        }
        pyresult.set_item("size", stats.size).unwrap();
        pyresult.set_item("usage", stats.usage).unwrap();
        if !stats.errors.is_empty() {
            pyresult.set_item("errors", stats.errors.to_vec()).unwrap();
        }
    }
    Ok(pyresult.into())
}

#[pyclass]
#[derive(Debug)]
pub struct Count {
    // Options
    root_path: String,
    skip_hidden: bool,
    metadata: bool,
    metadata_ext: bool,
    max_depth: usize,
    // Results
    statistics: Arc<Mutex<Statistics>>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Option<Weak<AtomicBool>>,
    exit_called: bool,
}

#[pymethods]
impl Count {
    #[new]
    fn __new__(
        obj: &PyRawObject,
        root_path: &str,
        skip_hidden: Option<bool>,
        metadata: Option<bool>,
        metadata_ext: Option<bool>,
        max_depth: Option<usize>,
    ) {
        obj.init(Count {
            root_path: String::from(root_path),
            skip_hidden: skip_hidden.unwrap_or(false),
            metadata: metadata.unwrap_or(false),
            metadata_ext: metadata_ext.unwrap_or(false),
            max_depth: max_depth.unwrap_or(::std::usize::MAX),
            statistics: Arc::new(Mutex::new(Statistics { 
                dirs: 0,
                files: 0,
                slinks: 0,
                hlinks: 0,
                devices: 0,
                pipes: 0,
                size: 0,
                usage: 0,
                errors: Vec::new(),
            })),
            thr: None,
            alive: None,
            exit_called: false,
        });
    }

    #[getter]
    fn statistics(&self) -> PyResult<Statistics> {
       Ok(Arc::clone(&self.statistics).lock().unwrap().clone())
    }

    fn collect(&self) -> PyResult<Statistics> {
        counter(self.root_path.clone(),
                self.skip_hidden, self.metadata, self.metadata_ext, self.max_depth,
                self.statistics.clone(), None);
        Ok(Arc::clone(&self.statistics).lock().unwrap().clone())
    }

    fn start(&mut self) -> PyResult<bool> {
        if self.thr.is_some() {
            return Err(exceptions::RuntimeError::py_err("Thread already running"))
        }
        let root_path = String::from(&self.root_path);
        let skip_hidden = self.skip_hidden;
        let metadata = self.metadata;
        let metadata_ext = self.metadata_ext;
        let max_depth = self.max_depth;
        let statistics = self.statistics.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            counter(root_path,
                    skip_hidden, metadata, metadata_ext, max_depth,
                    statistics, Some(alive))
        }));
        Ok(true)
    }

    fn stop(&mut self) -> PyResult<bool> {
        if self.thr.is_none() {
            return Err(exceptions::RuntimeError::py_err("Thread not running"))
        }
        match &self.alive {
            Some(alive) => match alive.upgrade() {
                Some(alive) => (*alive).store(false, Ordering::Relaxed),
                None => return Ok(false),
            },
            None => {},
        }
        self.thr.take().map(thread::JoinHandle::join);
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
impl pyo3::class::PyObjectProtocol for Count {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for Count {
    fn __enter__(&'p mut self) -> PyResult<()> {
        let root_path = String::from(&self.root_path);
        let skip_hidden = self.skip_hidden;
        let metadata = self.metadata;
        let metadata_ext = self.metadata_ext;
        let max_depth = self.max_depth;
        let statistics = self.statistics.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            counter(root_path,
                    skip_hidden, metadata, metadata_ext, max_depth,
                    statistics, Some(alive))
        }));
        Ok(())
    }

    fn __exit__(
        &mut self,
        ty: Option<&'p PyType>,
        _value: Option<&'p PyAny>,
        _traceback: Option<&'p PyAny>,
    ) -> PyResult<bool> {
        match &self.alive {
            Some(alive) => match alive.upgrade() {
                Some(alive) => (*alive).store(false, Ordering::Relaxed),
                None => return Ok(false),
            },
            None => {},
        }
        self.thr.take().map(thread::JoinHandle::join);
        let gil = GILGuard::acquire();
        self.exit_called = true;
        if ty == Some(gil.python().get_type::<ValueError>()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[pymodule(count)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Count>()?;
    m.add_wrapped(wrap_pyfunction!(count))?;
    Ok(())
}
