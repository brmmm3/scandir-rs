use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::Instant;

use crossbeam_channel as channel;
#[cfg(unix)]
use expanduser::expanduser;
use jwalk::WalkDir;

use pyo3::exceptions::{self, ValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple, PyType};
use pyo3::{wrap_pyfunction, PyContextProtocol, PyIterProtocol, Python};

use crate::def::*;

fn update_toc(
    entry: &std::result::Result<jwalk::core::dir_entry::DirEntry<()>, std::io::Error>,
    toc: &mut Toc,
) {
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
                toc.other.push(key.to_str().unwrap().to_string());
            }
        }
        Err(e) => toc.errors.push(e.to_string()), // TODO: Need to fetch failed path from somewhere
    }
}

pub fn rs_toc(
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    toc: Arc<Mutex<Toc>>,
    duration: Option<Arc<AtomicU64>>,
    alive: Option<Arc<AtomicBool>>,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    let start_time = Instant::now();
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    }
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
    {
        let mut toc_locked = toc.lock().unwrap();
        update_toc(&entry, toc_locked.deref_mut());
        match &alive {
            Some(a) => {
                if !a.load(Ordering::Relaxed) {
                    break;
                }
            }
            None => {}
        }
    }
    match &duration {
        Some(d) => {
            let dt = start_time.elapsed().as_millis() as f64;
            d.store(dt.to_bits(), Ordering::Relaxed);
        }
        None => {}
    }
}

pub fn rs_toc_iter(
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    duration: Option<Arc<AtomicU64>>,
    alive: Option<Arc<AtomicBool>>,
    tx: channel::Sender<WalkResult>,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    }
    let start_time = Instant::now();
    let mut toc = Toc {
        dirs: Vec::new(),
        files: Vec::new(),
        symlinks: Vec::new(),
        other: Vec::new(),
        errors: Vec::new(),
    };
    let mut send = false;
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
    {
        update_toc(&entry, &mut toc);
        if tx.is_empty() {
            if tx.send(WalkResult::Toc(toc)).is_err() {
                return;
            }
            toc = Toc {
                dirs: Vec::new(),
                files: Vec::new(),
                symlinks: Vec::new(),
                other: Vec::new(),
                errors: Vec::new(),
            };
            send = false;
        } else {
            send = true;
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
    if send {
        let _ = tx.send(WalkResult::Toc(toc));
    }
    match &duration {
        Some(d) => {
            let dt = start_time.elapsed().as_millis() as f64;
            d.store(dt.to_bits(), Ordering::Relaxed);
        }
        None => {}
    }
}

pub fn rs_walk_iter(
    root_path: String,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    duration: Option<Arc<AtomicU64>>,
    alive: Option<Arc<AtomicBool>>,
    tx: channel::Sender<WalkResult>,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    }
    let start_time = Instant::now();
    let mut list: Vec<String> = Vec::new();
    let mut map: HashMap<String, Toc> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
    {
        match &entry {
            Ok(v) => {
                let file_type = v.file_type_result.as_ref().unwrap();
                let dir = v.parent_path.to_str().unwrap().to_string();
                let file_name = v.file_name.clone().into_string().unwrap();
                if file_type.is_symlink() {
                    map.get_mut(&dir).unwrap().symlinks.push(file_name);
                } else if file_type.is_dir() {
                    let path = v.path().to_str().unwrap().to_string();
                    list.push(path.clone());
                    map.insert(
                        path,
                        Toc {
                            dirs: Vec::new(),
                            files: Vec::new(),
                            symlinks: Vec::new(),
                            other: Vec::new(),
                            errors: Vec::new(),
                        },
                    );
                    match map.get_mut(&dir) {
                        Some(toc) => toc.dirs.push(file_name),
                        None => {}
                    }
                } else if file_type.is_file() {
                    map.get_mut(&dir).unwrap().files.push(file_name);
                } else {
                    map.get_mut(&dir).unwrap().other.push(file_name);
                }
            }
            Err(e) => errors.push(e.to_string()), // TODO: Need to fetch failed path from somewhere
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
    match &duration {
        Some(d) => {
            let dt = start_time.elapsed().as_millis() as f64;
            d.store(dt.to_bits(), Ordering::Relaxed);
        }
        None => {}
    }
    for key in list {
        if tx
            .send(WalkResult::WalkEntry(WalkEntry {
                path: key.clone(),
                toc: map.get(&key).unwrap().clone(),
            }))
            .is_err()
        {
            break;
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
        other: Vec::new(),
        errors: Vec::new(),
    }));
    let toc_cloned = toc.clone();
    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        rs_toc(
            root_path,
            sorted.unwrap_or(false),
            skip_hidden.unwrap_or(false),
            max_depth.unwrap_or(::std::usize::MAX),
            toc_cloned,
            None,
            None,
        );
        Ok(())
    });
    match rc {
        Err(e) => return Err(exceptions::RuntimeError::py_err(e.to_string())),
        _ => (),
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
    return_type: u8,
    // Results
    toc: Arc<Mutex<Toc>>,
    duration: Arc<AtomicU64>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Option<Weak<AtomicBool>>,
    rx: Option<channel::Receiver<WalkResult>>,
    has_results: bool,
}

impl Walk {
    fn rs_init(&self) {
        let mut toc_locked = self.toc.lock().unwrap();
        toc_locked.dirs.clear();
        toc_locked.files.clear();
        toc_locked.symlinks.clear();
        toc_locked.other.clear();
        toc_locked.errors.clear();
    }

    fn rs_collect(&mut self) {
        rs_toc(
            self.root_path.clone(),
            self.sorted,
            self.skip_hidden,
            self.max_depth,
            self.toc.clone(),
            Some(self.duration.clone()),
            None,
        );
        self.has_results = true;
    }

    fn rs_start(&mut self, tx: Option<channel::Sender<WalkResult>>) -> bool {
        if self.thr.is_some() {
            return false;
        }
        if self.has_results {
            self.rs_init();
        }
        let root_path = String::from(&self.root_path);
        let sorted = self.sorted;
        let skip_hidden = self.skip_hidden;
        let max_depth = self.max_depth;
        let toc = self.toc.clone();
        let duration = self.duration.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        if tx.is_none() {
            self.thr = Some(thread::spawn(move || {
                rs_toc(
                    root_path,
                    sorted,
                    skip_hidden,
                    max_depth,
                    toc,
                    Some(duration),
                    Some(alive),
                )
            }));
        } else {
            if self.return_type == RETURN_TYPE_BASE {
                self.thr = Some(thread::spawn(move || {
                    rs_toc_iter(
                        root_path,
                        sorted,
                        skip_hidden,
                        max_depth,
                        Some(duration),
                        Some(alive),
                        tx.unwrap(),
                    )
                }));
            } else {
                self.thr = Some(thread::spawn(move || {
                    rs_walk_iter(
                        root_path,
                        sorted,
                        skip_hidden,
                        max_depth,
                        Some(duration),
                        Some(alive),
                        tx.unwrap(),
                    )
                }));
            }
        }
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
        return_type: Option<u8>,
    ) {
        obj.init(Walk {
            root_path: String::from(root_path),
            sorted: sorted.unwrap_or(false),
            skip_hidden: skip_hidden.unwrap_or(false),
            max_depth: max_depth.unwrap_or(::std::usize::MAX),
            return_type: return_type.unwrap_or(RETURN_TYPE_WALK),
            toc: Arc::new(Mutex::new(Toc {
                dirs: Vec::new(),
                files: Vec::new(),
                symlinks: Vec::new(),
                other: Vec::new(),
                errors: Vec::new(),
            })),
            duration: Arc::new(AtomicU64::new(0)),
            thr: None,
            alive: None,
            rx: None,
            has_results: false,
        });
    }

    #[getter]
    fn toc(&self) -> PyResult<Toc> {
        let mut toc_locked = self.toc.lock().unwrap();
        let tmp_toc = toc_locked.clone();
        toc_locked.deref_mut().clear();
        Ok(tmp_toc)
    }

    #[getter]
    fn duration(&self) -> PyResult<f64> {
        Ok(f64::from_bits(self.duration.load(Ordering::Relaxed)) * 0.001)
    }

    fn has_results(&self) -> PyResult<bool> {
        Ok(self.has_results)
    }

    fn as_dict(&self) -> PyResult<PyObject> {
        let gil = GILGuard::acquire();
        let mut toc_locked = self.toc.lock().unwrap();
        let pyresult = PyDict::new(gil.python());
        if !toc_locked.dirs.is_empty() {
            pyresult.set_item("dirs", toc_locked.dirs.to_vec()).unwrap();
        }
        if !toc_locked.files.is_empty() {
            pyresult
                .set_item("files", toc_locked.files.to_vec())
                .unwrap();
        }
        if !toc_locked.symlinks.is_empty() {
            pyresult
                .set_item("symlinks", toc_locked.symlinks.to_vec())
                .unwrap();
        }
        if !toc_locked.other.is_empty() {
            pyresult
                .set_item("other", toc_locked.other.to_vec())
                .unwrap();
        }
        if !toc_locked.errors.is_empty() {
            pyresult
                .set_item("errors", toc_locked.errors.to_vec())
                .unwrap();
        }
        toc_locked.deref_mut().clear();
        Ok(pyresult.into())
    }

    fn collect(&mut self) -> PyResult<Toc> {
        self.rs_collect();
        Ok(Arc::clone(&self.toc).lock().unwrap().clone())
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
impl<'p> PyIterProtocol for Walk {
    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<Walk>> {
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
        match &slf.rx {
            Some(rx) => match rx.recv() {
                Ok(val) => match val {
                    WalkResult::Toc(toc) => Ok(Some(toc.to_object(gil.python()))),
                    WalkResult::WalkEntry(mut entry) => {
                        if slf.return_type == RETURN_TYPE_EXT {
                            Ok(Some(entry.to_object(gil.python())))
                        } else {
                            let py = gil.python();
                            let mut files = entry.toc.files.to_vec();
                            files.append(&mut entry.toc.symlinks);
                            files.append(&mut entry.toc.other);
                            Ok(Some(
                                PyTuple::new(
                                    py,
                                    &[
                                        entry.path.to_object(py),
                                        entry.toc.dirs.to_object(py),
                                        files.to_object(py),
                                    ],
                                )
                                .into(),
                            ))
                        }
                    }
                },
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
