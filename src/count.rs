use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::Instant;

use pyo3::exceptions::{self, ValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyType};
use pyo3::{wrap_pyfunction, PyContextProtocol, Python};

use crate::common::{create_filter, expand_path, walk};
use crate::def::*;

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
    pub duration: f64,
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Statistics {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pymethods]
impl Statistics {
    #[getter]
    fn dirs(&self) -> PyResult<u32> {
        Ok(self.dirs)
    }

    #[getter]
    fn files(&self) -> PyResult<u32> {
        Ok(self.files)
    }

    #[getter]
    fn slinks(&self) -> PyResult<u32> {
        Ok(self.slinks)
    }

    #[getter]
    fn hlinks(&self) -> PyResult<u32> {
        Ok(self.hlinks)
    }

    #[getter]
    fn devices(&self) -> PyResult<u32> {
        Ok(self.devices)
    }

    #[getter]
    fn pipes(&self) -> PyResult<u32> {
        Ok(self.pipes)
    }

    #[getter]
    fn size(&self) -> PyResult<u64> {
        Ok(self.size)
    }

    #[getter]
    fn usage(&self) -> PyResult<u64> {
        Ok(self.usage)
    }

    #[getter]
    fn errors(&self) -> PyResult<Vec<String>> {
        Ok(self.errors.to_vec())
    }

    #[getter]
    fn duration(&self) -> PyResult<f64> {
        Ok(self.duration)
    }
}

fn rs_count(
    root_path: &String,
    skip_hidden: bool,
    extended: bool, // If true: Count also hardlinks, devices, pipes, size and usage
    mut max_depth: usize,
    filter: Option<Filter>,
    statistics: &Arc<Mutex<Statistics>>,
    alive: Option<Arc<AtomicBool>>,
) {
    #[cfg(unix)]
    let root_path = expand_path(&root_path);
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
    for entry in walk(
        &root_path,
        false,
        skip_hidden,
        max_depth,
        filter,
        match extended {
            false => RETURN_TYPE_FAST,
            true => RETURN_TYPE_EXT,
        },
    ) {
        match &entry {
            Ok(v) => {
                let file_type = v.file_type_result.as_ref().unwrap();
                if file_type.is_dir() {
                    dirs += 1;
                } else if file_type.is_file() {
                    files += 1;
                } else if file_type.is_symlink() {
                    slinks += 1;
                }
                if v.metadata_result.is_some() {
                    let metadata_ext = v.ext.as_ref();
                    if metadata_ext.is_some() {
                        let metadata_ext = metadata_ext.unwrap().as_ref().unwrap();
                        if metadata_ext.nlink > 1 {
                            if file_indexes.contains(&metadata_ext.ino) {
                                hlinks += 1;
                            } else {
                                file_indexes.insert(metadata_ext.ino);
                            }
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
            Err(e) => errors.push(e.to_string()), // TODO: Need to fetch failed path from somewhere
        }
        cnt += 1;
        if (cnt >= 1000) || (update_time.elapsed().as_millis() >= 10) {
            let mut stats_locked = statistics.lock().unwrap();
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
        match &alive {
            Some(a) => {
                if !a.load(Ordering::Relaxed) {
                    break;
                }
            }
            None => {}
        }
    }
    let mut stats_locked = statistics.lock().unwrap();
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
    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        rs_count(
            &root_path,
            skip_hidden.unwrap_or(false),
            extended.unwrap_or(false),
            max_depth.unwrap_or(::std::usize::MAX),
            filter,
            &stats_cloned,
            None,
        );
        Ok(())
    });
    match rc {
        Err(e) => return Err(exceptions::RuntimeError::py_err(e.to_string())),
        _ => (),
    }
    let stats_cloned = statistics.lock().unwrap().clone();
    Ok(stats_cloned.into())
}

#[pyclass]
#[derive(Debug)]
pub struct Count {
    // Options
    root_path: String,
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
        let root_path = String::from(&self.root_path);
        let skip_hidden = self.skip_hidden;
        let extended = self.extended;
        let max_depth = self.max_depth;
        let filter = self.filter.clone();
        let statistics = self.statistics.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            rs_count(
                &root_path,
                skip_hidden,
                extended,
                max_depth,
                filter,
                &statistics,
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
        obj: &PyRawObject,
        root_path: &str,
        skip_hidden: Option<bool>,
        extended: Option<bool>,
        max_depth: Option<usize>,
    ) {
        obj.init(Count {
            root_path: String::from(root_path),
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
        });
    }

    #[getter]
    fn statistics(&self) -> PyResult<Statistics> {
        Ok(Arc::clone(&self.statistics).lock().unwrap().clone())
    }

    fn has_results(&self) -> PyResult<bool> {
        Ok(self.has_results)
    }

    fn as_dict(&self) -> PyResult<PyObject> {
        let gil = GILGuard::acquire();
        let stats_locked = self.statistics.lock().unwrap();
        let pyresult = PyDict::new(gil.python());
        if stats_locked.dirs > 0 {
            pyresult.set_item("dirs", stats_locked.dirs).unwrap();
        }
        if stats_locked.files > 0 {
            pyresult.set_item("files", stats_locked.files).unwrap();
        }
        if stats_locked.slinks > 0 {
            pyresult.set_item("slinks", stats_locked.slinks).unwrap();
        }
        if stats_locked.hlinks > 0 {
            pyresult.set_item("hlinks", stats_locked.hlinks).unwrap();
        }
        if stats_locked.devices > 0 {
            pyresult.set_item("devices", stats_locked.devices).unwrap();
        }
        if stats_locked.pipes > 0 {
            pyresult.set_item("pipes", stats_locked.pipes).unwrap();
        }
        if stats_locked.size > 0 {
            pyresult.set_item("size", stats_locked.size).unwrap();
        }
        if stats_locked.usage > 0 {
            pyresult.set_item("usage", stats_locked.usage).unwrap();
        }
        if !stats_locked.errors.is_empty() {
            pyresult
                .set_item("errors", stats_locked.errors.to_vec())
                .unwrap();
        }
        pyresult
            .set_item("duration", stats_locked.duration().unwrap())
            .unwrap();
        Ok(pyresult.to_object(gil.python()))
    }

    fn collect(&mut self) -> PyResult<Statistics> {
        let gil = GILGuard::acquire();
        let rc: std::result::Result<(), std::io::Error> = gil.python().allow_threads(|| {
            rs_count(
                &self.root_path,
                self.skip_hidden,
                self.extended,
                self.max_depth,
                self.filter.clone(),
                &self.statistics,
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
