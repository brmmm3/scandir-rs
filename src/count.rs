use std::collections::HashSet;
use std::fs;
use std::fs::Metadata;
use std::io::{Error, ErrorKind};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::Instant;

use pyo3::exceptions::{self, ValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyType};
use pyo3::{wrap_pyfunction, PyContextProtocol, Python};

use jwalk::WalkDirGeneric;

use crate::common::check_and_expand_path;
use crate::common::{create_filter, filter_children};
use crate::def::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Statistics {
    #[pyo3(get)]
    pub dirs: u32,
    #[pyo3(get)]
    pub files: u32,
    #[pyo3(get)]
    pub slinks: u32,
    #[pyo3(get)]
    pub hlinks: u32,
    #[pyo3(get)]
    pub devices: u32,
    #[pyo3(get)]
    pub pipes: u32,
    #[pyo3(get)]
    pub size: u64,
    #[pyo3(get)]
    pub usage: u64,
    #[pyo3(get)]
    pub errors: Vec<String>,
    #[pyo3(get)]
    pub duration: f64,
}

#[pymethods]
impl Statistics {
    fn as_dict(&self, duration: Option<bool>) -> PyResult<PyObject> {
        let gil = GILGuard::acquire();
        let pyresult = PyDict::new(gil.python());
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
        if duration.unwrap_or(false) {
            pyresult.set_item("duration", self.duration).unwrap();
        }
        Ok(pyresult.to_object(gil.python()))
    }
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Statistics {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

fn rs_count(
    root_path: PathBuf,
    skip_hidden: bool,
    extended: bool, // If true: Count also hardlinks, devices, pipes, size and usage
    mut max_depth: usize,
    filter: Option<Filter>,
    statistics: Option<Arc<Mutex<Statistics>>>,
    alive: Option<Arc<AtomicBool>>,
) {
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
    let start_time = Instant::now();
    let mut update_time = Instant::now();
    let mut file_indexes: HashSet<u64> = HashSet::new();
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    }
    let root_path_len = root_path.to_string_lossy().len() + 1;
    for entry in WalkDirGeneric::<((), Option<Result<Metadata, Error>>)>::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(false)
        .max_depth(max_depth)
        .process_read_dir(move |_, children| {
            match &alive {
                Some(a) => {
                    if !a.load(Ordering::Relaxed) {
                        return;
                    }
                }
                None => {}
            }
            filter_children(children, &filter, root_path_len);
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    if extended {
                        dir_entry.client_state = Some(fs::metadata(dir_entry.path()));
                    }
                }
            });
        })
    {
        match &entry {
            Ok(v) => {
                let file_type = v.file_type;
                if file_type.is_dir() {
                    dirs += 1;
                } else if file_type.is_file() {
                    files += 1;
                } else if file_type.is_symlink() {
                    slinks += 1;
                }
                if let Some(cs) = &v.client_state {
                    if let Ok(metadata) = cs {
                        #[cfg(unix)]
                        {
                            if metadata.nlink() > 1 {
                                if file_indexes.contains(&metadata.ino()) {
                                    hlinks += 1;
                                } else {
                                    file_indexes.insert(metadata.ino());
                                }
                            }
                            let file_size = metadata.size();
                            let mut blocks = file_size >> 12;
                            if blocks << 12 < file_size {
                                blocks += 1;
                            }
                            usage += blocks << 12;
                            size += file_size;
                            if metadata.rdev() > 0 {
                                devices += 1;
                            }
                            if (metadata.mode() & 4096) != 0 {
                                pipes += 1;
                            }
                        }
                        #[cfg(windows)]
                        {
                            if let Some(nlink) = metadata.number_of_links() {
                                if nlink > 1 {
                                    if let Some(ino) = metadata.file_index() {
                                        if file_indexes.contains(&ino) {
                                            hlinks += 1;
                                        } else {
                                            file_indexes.insert(ino);
                                        }
                                    }
                                }
                            }
                            let file_size = metadata.file_size();
                            let mut blocks = file_size >> 12;
                            if blocks << 12 < file_size {
                                blocks += 1;
                            }
                            usage += blocks << 12;
                            size += file_size;
                        }
                    }
                }
                cnt += 1;
                if (cnt >= 1000) || (update_time.elapsed().as_millis() >= 10) {
                    match &statistics {
                        Some(stats) => {
                            let mut stats_locked = stats.lock().unwrap();
                            stats_locked.dirs = dirs;
                            stats_locked.files = files;
                            stats_locked.slinks = slinks;
                            stats_locked.hlinks = hlinks;
                            stats_locked.size = size;
                            stats_locked.usage = usage;
                            if stats_locked.errors.len() < errors.len() {
                                stats_locked.errors.extend_from_slice(&errors);
                                errors.clear();
                            }
                            stats_locked.duration = start_time.elapsed().as_millis() as f64 * 0.001;
                            #[cfg(unix)]
                            {
                                stats_locked.devices = devices;
                                stats_locked.pipes = pipes;
                            }
                            cnt = 0;
                            update_time = Instant::now();
                        }
                        None => {}
                    }
                }
            }
            Err(e) => errors.push(e.to_string()), // TODO: Need to fetch failed path from somewhere
        }
    }
    match &statistics {
        Some(stats) => {
            let mut stats_locked = stats.lock().unwrap();
            stats_locked.dirs = dirs;
            stats_locked.files = files;
            stats_locked.slinks = slinks;
            stats_locked.hlinks = hlinks;
            stats_locked.size = size;
            stats_locked.usage = usage;
            if stats_locked.errors.len() < errors.len() {
                stats_locked.errors.extend_from_slice(&errors);
                errors.clear();
            }
            stats_locked.duration = start_time.elapsed().as_millis() as f64 * 0.001;
            #[cfg(unix)]
            {
                stats_locked.devices = devices;
                stats_locked.pipes = pipes;
            }
        }
        None => {}
    }
}

#[pyfunction]
pub fn count(
    py: Python,
    root_path: String,
    skip_hidden: Option<bool>,
    extended: Option<bool>,
    max_depth: Option<usize>,
    dir_include: Option<Vec<String>>,
    dir_exclude: Option<Vec<String>>,
    file_include: Option<Vec<String>>,
    file_exclude: Option<Vec<String>>,
    case_sensitive: Option<bool>,
) -> PyResult<Statistics> {
    let filter = match create_filter(
        dir_include,
        dir_exclude,
        file_include,
        file_exclude,
        case_sensitive,
    ) {
        Ok(f) => f,
        Err(e) => return Err(exceptions::ValueError::py_err(e.to_string())),
    };
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
        duration: 0.0,
    }));
    let stats_cloned = statistics.clone();
    let root_path = match check_and_expand_path(&root_path) {
        Ok(p) => p,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                return Err(exceptions::FileNotFoundError::py_err(e.to_string()))
            }
            _ => return Err(exceptions::Exception::py_err(e.to_string())),
        },
    };
    let rc: Result<(), Error> = py.allow_threads(|| {
        rs_count(
            root_path,
            skip_hidden.unwrap_or(false),
            extended.unwrap_or(false),
            max_depth.unwrap_or(::std::usize::MAX),
            filter,
            Some(stats_cloned),
            None,
        );
        Ok(())
    });
    if let Err(e) = rc {
        return Err(exceptions::RuntimeError::py_err(e.to_string()));
    }
    let stats_cloned = statistics.lock().unwrap().clone();
    Ok(stats_cloned.into())
}

#[pyclass]
#[derive(Debug)]
pub struct Count {
    // Options
    root_path: PathBuf,
    skip_hidden: bool,
    extended: bool,
    max_depth: usize,
    filter: Option<Filter>,
    // Results
    statistics: Arc<Mutex<Statistics>>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Option<Weak<AtomicBool>>,
    has_results: bool,
}

impl Count {
    fn rs_init(&self) {
        let mut stats_locked = self.statistics.lock().unwrap();
        stats_locked.dirs = 0;
        stats_locked.files = 0;
        stats_locked.slinks = 0;
        stats_locked.hlinks = 0;
        stats_locked.size = 0;
        stats_locked.usage = 0;
        stats_locked.errors.clear();
        #[cfg(unix)]
        {
            stats_locked.devices = 0;
            stats_locked.pipes = 0;
        }
    }

    fn rs_start(&mut self) -> bool {
        if self.thr.is_some() {
            return false;
        }
        if self.has_results {
            self.rs_init();
        }
        let root_path = self.root_path.clone();
        let skip_hidden = self.skip_hidden;
        let extended = self.extended;
        let max_depth = self.max_depth;
        let filter = self.filter.clone();
        let statistics = self.statistics.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            rs_count(
                root_path,
                skip_hidden,
                extended,
                max_depth,
                filter,
                Some(statistics),
                Some(alive),
            )
        }));
        true
    }

    fn rs_stop(&mut self) -> bool {
        match &self.alive {
            Some(alive) => match alive.upgrade() {
                Some(alive) => (*alive).store(false, Ordering::Relaxed),
                None => {}
            },
            None => {}
        }
        if self.thr.is_none() {
            return false;
        }
        self.thr.take().map(thread::JoinHandle::join);
        self.has_results = true;
        true
    }
}

#[pymethods]
impl Count {
    #[new]
    fn __new__(
        root_path: &str,
        skip_hidden: Option<bool>,
        extended: Option<bool>,
        max_depth: Option<usize>,
    ) -> PyResult<Self> {
        let root_path = match check_and_expand_path(&root_path) {
            Ok(p) => p,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    return Err(exceptions::FileNotFoundError::py_err(e.to_string()))
                }
                _ => return Err(exceptions::Exception::py_err(e.to_string())),
            },
        };
        Ok(Count {
            root_path: root_path,
            skip_hidden: skip_hidden.unwrap_or(false),
            extended: extended.unwrap_or(false),
            max_depth: max_depth.unwrap_or(::std::usize::MAX),
            filter: None,
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
                duration: 0.0,
            })),
            thr: None,
            alive: None,
            has_results: false,
        })
    }

    #[getter]
    fn statistics(&self) -> PyResult<Statistics> {
        Ok(Arc::clone(&self.statistics).lock().unwrap().clone())
    }

    fn has_results(&self) -> PyResult<bool> {
        Ok(self.has_results)
    }

    fn as_dict(&self, duration: Option<bool>) -> PyResult<PyObject> {
        self.statistics.lock().unwrap().as_dict(duration)
    }

    fn collect(&mut self) -> PyResult<Statistics> {
        let gil = GILGuard::acquire();
        let rc: Result<(), Error> = gil.python().allow_threads(|| {
            rs_count(
                self.root_path.clone(),
                self.skip_hidden,
                self.extended,
                self.max_depth,
                self.filter.clone(),
                Some(self.statistics.clone()),
                None,
            );
            Ok(())
        });
        match rc {
            Err(e) => return Err(exceptions::RuntimeError::py_err(e.to_string())),
            _ => (),
        }
        self.has_results = true;
        Ok(Arc::clone(&self.statistics).lock().unwrap().clone())
    }

    fn start(&mut self) -> PyResult<bool> {
        if !self.rs_start() {
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
impl pyo3::class::PyObjectProtocol for Count {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for Count {
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
            return Ok(false);
        }
        if ty == Some(GILGuard::acquire().python().get_type::<ValueError>()) {
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
