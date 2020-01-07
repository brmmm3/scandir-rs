use std::time::Instant;
use std::thread;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::ops::DerefMut;

use jwalk::WalkDir;
#[cfg(unix)]
use expanduser::expanduser;
use crossbeam_channel as channel;

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
    pub duration: f64,
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
    fn unknown(&self) -> PyResult<Vec<String>> {
        Ok(self.unknown.to_vec())
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

fn update_toc(entry: &std::result::Result<jwalk::core::dir_entry::DirEntry<()>, std::io::Error>,
              toc: &mut Toc) {
    match &entry {
        Ok(v) => {
            let file_type = v.file_type_result.as_ref().unwrap();
            let mut key = v.parent_path.to_path_buf();
            key.push(v.file_name.clone().into_string().unwrap());
            if file_type.is_symlink() {
                toc.symlinks.push(key.to_str().unwrap().to_string());
            } else if file_type.is_dir() {
                toc.dirs.push(key.to_str().unwrap().to_string());
            } else if file_type.is_file() {
                toc.files.push(key.to_str().unwrap().to_string());
            } else {
                toc.unknown.push(key.to_str().unwrap().to_string());
            }
        }
        Err(e) => toc.errors.push(e.to_string())  // TODO: Need to fetch failed path from somewhere
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
    let start_time = Instant::now();
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
    {
        let mut toc_locked = toc.lock().unwrap();
        update_toc(&entry, toc_locked.deref_mut());
        toc_locked.duration = start_time.elapsed().as_millis() as f64 * 0.001;
        match &alive {
            Some(a) => if !a.load(Ordering::Relaxed) {
                break;
            },
            None => {},
        }
    }
    toc.lock().unwrap().duration = start_time.elapsed().as_millis() as f64 * 0.001;
}

pub fn rs_toc_iter(
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    max_depth: usize,
    alive: Option<Arc<AtomicBool>>,
    tx: Option<channel::Sender<Toc>>
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    let start_time = Instant::now();
    let mut toc = Toc {
        dirs: Vec::new(),
        files: Vec::new(),
        symlinks: Vec::new(),
        unknown: Vec::new(),
        errors: Vec::new(),
        duration: 0.0,
    };
    let mut send = false;
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
    {
        update_toc(&entry, &mut toc);
        match &tx {
            Some(tx) => {
                if tx.is_empty() {
                    tx.send(toc).unwrap();
                    toc = Toc {
                        dirs: Vec::new(),
                        files: Vec::new(),
                        symlinks: Vec::new(),
                        unknown: Vec::new(),
                        errors: Vec::new(),
                        duration: start_time.elapsed().as_millis() as f64 * 0.001,
                    };
                    send = false;
                } else {
                    send = true;
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
    toc.duration = start_time.elapsed().as_millis() as f64 * 0.001;
    if send {
        match &tx {
            Some(tx) => tx.send(toc).unwrap(),
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
        duration: 0.0,
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
    rx: Option<channel::Receiver<Toc>>,
    has_results: bool,
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

    fn rs_start(&mut self,
        tx: Option<channel::Sender<Toc>>,
    ) -> bool {
        if self.thr.is_some() {
            return false
        }
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
        if tx.is_none() {
            self.thr = Some(thread::spawn(move || {
                rs_toc(root_path,
                       sorted, skip_hidden, max_depth,
                       toc, Some(alive))
            }));
        } else {
            self.thr = Some(thread::spawn(move || {
                rs_toc_iter(root_path,
                            sorted, skip_hidden, max_depth,
                            Some(alive), tx)
            }));
        }
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
        self.has_results = true;
        true
    }

    fn rs_busy(&self) -> bool {
        match &self.alive {
            Some(alive) => match alive.upgrade() {
                Some(_) => true,
                None => return false,
            },
            None => false,
        }
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
                duration: 0.0,
            })),
            thr: None,
            alive: None,
            rx: None,
            has_results: false,
        });
    }

    #[getter]
    fn toc(&self) -> PyResult<Toc> {
       Ok(Arc::clone(&self.toc).lock().unwrap().clone())
    }

    fn has_results(&self) -> PyResult<bool> {
        Ok(self.has_results)
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
        pyresult.set_item("duration", toc_locked.duration().unwrap()).unwrap();
        Ok(pyresult.into())
    }

    fn list(&mut self) -> PyResult<Toc> {
        rs_toc(self.root_path.clone(),
               self.sorted, self.skip_hidden, self.max_depth,
               self.toc.clone(), None);
        self.has_results = true;
        Ok(Arc::clone(&self.toc).lock().unwrap().clone())
    }

    fn start(&mut self) -> PyResult<bool> {
        if !self.rs_start(None) {
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
        Ok(self.rs_busy())
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
        if !self.rs_start(None) {
            return Err(exceptions::RuntimeError::py_err("Thread already running"))
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
    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<Walk>> {
        if slf.thr.is_some() {
            return Err(exceptions::RuntimeError::py_err("Thread already running"))
        }
        let (tx, rx) = channel::unbounded();
        slf.rx = Some(rx);
        slf.rs_start(Some(tx));
        Ok(slf.into())
    }

    fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<Toc>> {
        match &slf.rx {
            Some(rx) => match rx.recv() {
                Ok(val) => Ok(Some(val)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }
}

#[pymodule(walk)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Walk>()?;
    m.add_wrapped(wrap_pyfunction!(toc))?;
    Ok(())
}
