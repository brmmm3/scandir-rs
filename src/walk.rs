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

#[pyclass]
#[derive(Debug, Clone)]
pub struct Toc {
    pub dirs: Vec<String>,
    pub files: Vec<String>,
    pub symlinks: Vec<String>,
    pub unknown: Vec<String>,
    pub errors: Vec<String>,
}

#[pyproto]
impl pyo3::class::PyObjectProtocol for Toc {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

pub fn rs_toc(
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    max_depth: usize,
    toc: Arc<Mutex<Toc>>,
    alive: Option<Arc<AtomicBool>>,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
    {
        match &entry {
            Ok(v) => {
                let file_type = v.file_type_result.as_ref().unwrap();
                let mut key = v.parent_path.to_path_buf();
                key.push(v.file_name.clone().into_string().unwrap());
                println!("{} {:?}", v.depth, key);
                let mut toc_locked = toc.lock().unwrap();
                if file_type.is_symlink() {
                    toc_locked.symlinks.push(key.to_str().unwrap().to_string());
                } else if file_type.is_dir() {
                    toc_locked.dirs.push(key.to_str().unwrap().to_string());
                } else if file_type.is_file() {
                    toc_locked.files.push(key.to_str().unwrap().to_string());
                } else {
                    toc_locked.unknown.push(key.to_str().unwrap().to_string());
                }
            }
            Err(e) => toc.lock().unwrap().errors.push(e.to_string())  // TODO: Need to fetch failed path from somewhere
        }
        match &alive {
            Some(a) => if !a.load(Ordering::Relaxed) {
                break;
            },
            None => {},
        }
    }
}

#[pyfunction]
pub fn toc(
    py: Python,
    root_path: String,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
    max_depth: Option<usize>,
) -> PyResult<Toc> {
    let toc = Arc::new(Mutex::new(Toc { 
        dirs: Vec::new(),
        files: Vec::new(),
        symlinks: Vec::new(),
        unknown: Vec::new(),
        errors: Vec::new(),
    }));
    let toc_cloned = toc.clone();
    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        rs_toc(root_path,
               sorted.unwrap_or(false),
               skip_hidden.unwrap_or(false),
               max_depth.unwrap_or(::std::usize::MAX),
               toc_cloned, None);
        Ok(())
    });
    match rc {
        Err(e) => return Err(exceptions::RuntimeError::py_err(e.to_string())),
        _ => ()
    }
    let toc_cloned = toc.lock().unwrap().clone();
    Ok(toc_cloned.into())
}

#[pyclass]
#[derive(Debug)]
pub struct Walk {
    // Options
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    max_depth: usize,
    // Results
    toc: Arc<Mutex<Toc>>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Option<Weak<AtomicBool>>,
    has_results: bool,
    start_time: Instant,
    duration: Duration,
}

impl Walk {
    fn rs_init(&self) {
        let mut toc_locked = self.toc.lock().unwrap();
        toc_locked.dirs.clear();
        toc_locked.files.clear();
        toc_locked.symlinks.clear();
        toc_locked.unknown.clear();
        toc_locked.errors.clear();
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
        let max_depth = self.max_depth;
        let toc = self.toc.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            rs_toc(root_path,
                   sorted, skip_hidden, max_depth,
                   toc, Some(alive))
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
impl Walk {
    #[new]
    fn __new__(
        obj: &PyRawObject,
        root_path: &str,
        sorted: Option<bool>,
        skip_hidden: Option<bool>,
        max_depth: Option<usize>,
    ) {
        obj.init(Walk {
            root_path: String::from(root_path),
            sorted: sorted.unwrap_or(false),
            skip_hidden: skip_hidden.unwrap_or(false),
            max_depth: max_depth.unwrap_or(::std::usize::MAX),
            toc: Arc::new(Mutex::new(Toc { 
                dirs: Vec::new(),
                files: Vec::new(),
                symlinks: Vec::new(),
                unknown: Vec::new(),
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
    fn toc(&self) -> PyResult<Toc> {
       Ok(Arc::clone(&self.toc).lock().unwrap().clone())
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
        let toc_locked = self.toc.lock().unwrap();
        let pyresult = PyDict::new(gil.python());
        if !toc_locked.dirs.is_empty() {
            pyresult.set_item("dirs", toc_locked.dirs.to_vec()).unwrap();
        }
        if !toc_locked.files.is_empty() {
            pyresult.set_item("files", toc_locked.files.to_vec()).unwrap();
        }
        if !toc_locked.symlinks.is_empty() {
            pyresult.set_item("symlinks", toc_locked.symlinks.to_vec()).unwrap();
        }
        if !toc_locked.unknown.is_empty() {
            pyresult.set_item("unknown", toc_locked.unknown.to_vec()).unwrap();
        }
        if !toc_locked.errors.is_empty() {
            pyresult.set_item("errors", toc_locked.errors.to_vec()).unwrap();
        }
        Ok(pyresult.into())
    }

    fn list(&mut self) -> PyResult<Toc> {
        self.start_time = Instant::now();
        rs_toc(self.root_path.clone(),
               self.sorted, self.skip_hidden, self.max_depth,
               self.toc.clone(), None);
        self.duration = self.start_time.elapsed();
        self.has_results = true;
        Ok(Arc::clone(&self.toc).lock().unwrap().clone())
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
impl pyo3::class::PyObjectProtocol for Walk {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for Walk {
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
impl<'p> PyIterProtocol for Walk {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Walk>> {
        Ok(slf.into())
    }

    fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<Toc>> {
        let toc_locked = slf.toc.lock().unwrap();
        if toc_locked.dirs.is_empty()
                && toc_locked.files.is_empty()
                && toc_locked.symlinks.is_empty()
                && toc_locked.unknown.is_empty()
                && toc_locked.errors.is_empty() {
            return Ok(None)
        }
        Ok(Some(toc_locked.clone()))
    }
}

#[pymodule(walk)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Walk>()?;
    m.add_wrapped(wrap_pyfunction!(toc))?;
    Ok(())
}
