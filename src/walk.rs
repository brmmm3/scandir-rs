use std::fmt::Debug;
use std::fs;
use std::io::{Error, ErrorKind};
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::Instant;

use crossbeam_channel as channel;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyType};
use pyo3::{wrap_pyfunction, Python};

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

fn update_toc(
    dir_entry: &jwalk::DirEntry<((), Option<Result<fs::Metadata, Error>>)>,
    toc: &mut Toc,
) {
    let file_type = dir_entry.file_type;
    let mut key = dir_entry.parent_path.to_path_buf();
    key.push(dir_entry.file_name.clone().into_string().unwrap());
    if file_type.is_symlink() {
        toc.symlinks.push(key.to_str().unwrap().to_string());
    } else if file_type.is_dir() {
        toc.dirs.push(key.to_str().unwrap().to_string());
    } else if file_type.is_file() {
        COUNT.store(COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
        toc.files.push(key.to_str().unwrap().to_string());
    } else {
        toc.other.push(key.to_str().unwrap().to_string());
    }
}

pub fn rs_toc(
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    filter: Option<Filter>,
    toc: Arc<Mutex<Toc>>,
    duration: Option<Arc<AtomicU64>>,
    alive: Option<Arc<AtomicBool>>,
) {
    BUSY.store(true, Ordering::Relaxed);
    let start_time = Instant::now();
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    }
    let root_path_len = root_path.to_string_lossy().len() + 1;
    for _ in WalkDirGeneric::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
        .process_read_dir(move |_, _, _, children| {
            match &alive {
                Some(a) => {
                    if !a.load(Ordering::Relaxed) {
                        BUSY.store(false, Ordering::Relaxed);
                        return;
                    }
                }
                None => {}
            }
            filter_children(children, &filter, root_path_len);
            if !children.is_empty() {
                let mut toc_locked = toc.lock().unwrap();
                children.iter_mut().for_each(|dir_entry_result| {
                    if let Ok(dir_entry) = dir_entry_result {
                        update_toc(&dir_entry, toc_locked.deref_mut());
                    }
                });
            }
        })
    {}
    match &duration {
        Some(d) => {
            let dt = start_time.elapsed().as_millis() as f64;
            d.store(dt.to_bits(), Ordering::Relaxed);
        }
        None => {}
    }
    BUSY.store(false, Ordering::Relaxed);
}

pub fn rs_toc_iter(
    root_path: Option<PathBuf>,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    filter: Option<Filter>,
    duration: Option<Arc<AtomicU64>>,
    alive: Option<Arc<AtomicBool>>,
    tx: channel::Sender<WalkResult>,
) {
    BUSY.store(true, Ordering::Relaxed);
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    }
    let start_time = Instant::now();
    let root_path = root_path.unwrap();
    let root_path_len = root_path.to_string_lossy().len() + 1;
    let tx_clone = tx.clone();
    for _ in WalkDirGeneric::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
        .process_read_dir(move |_, _, _, children| {
            match &alive {
                Some(a) => {
                    if !a.load(Ordering::Relaxed) {
                        BUSY.store(false, Ordering::Relaxed);
                        return;
                    }
                }
                None => {}
            }
            filter_children(children, &filter, root_path_len);
            if !children.is_empty() {
                let mut path: Option<String> = None;
                let mut toc = Toc {
                    dirs: Vec::new(),
                    files: Vec::new(),
                    symlinks: Vec::new(),
                    other: Vec::new(),
                    errors: Vec::new(),
                };
                children
                    .iter_mut()
                    .for_each(|dir_entry_result| match dir_entry_result {
                        Ok(dir_entry) => {
                            let file_type = dir_entry.file_type;
                            let file_name = dir_entry.file_name.clone().into_string().unwrap();
                            if path.is_none() {
                                path = Some(dir_entry.parent_path.to_str().unwrap().to_string());
                            }
                            if file_type.is_symlink() {
                                toc.symlinks.push(file_name);
                            } else if file_type.is_dir() {
                                toc.dirs.push(file_name);
                            } else if file_type.is_file() {
                                toc.files.push(file_name);
                            } else {
                                toc.other.push(file_name);
                            }
                        }
                        Err(e) => toc.errors.push(e.to_string()),
                    });
                tx_clone.send(WalkResult::Toc(toc.clone())).unwrap();
            }
        })
    {}
    match &duration {
        Some(d) => {
            let dt = start_time.elapsed().as_millis() as f64;
            d.store(dt.to_bits(), Ordering::Relaxed);
        }
        None => {}
    }
    BUSY.store(false, Ordering::Relaxed);
}

pub fn rs_walk_iter(
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    filter: Option<Filter>,
    return_type: u8,
    duration: Option<Arc<AtomicU64>>,
    alive: Option<Arc<AtomicBool>>,
    tx: channel::Sender<WalkResult>,
) {
    BUSY.store(true, Ordering::Relaxed);
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    }
    let start_time = Instant::now();
    let root_path_len = root_path.to_string_lossy().len() + 1;
    let tx_clone = tx.clone();
    for _ in WalkDirGeneric::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth)
        .process_read_dir(move |_, _, _, children| {
            match &alive {
                Some(a) => {
                    if !a.load(Ordering::Relaxed) {
                        BUSY.store(false, Ordering::Relaxed);
                        return;
                    }
                }
                None => {}
            }
            filter_children(children, &filter, root_path_len);
            if children.is_empty() {
                BUSY.store(false, Ordering::Relaxed);
                return;
            }
            let mut path: Option<String> = None;
            let mut toc = Toc {
                dirs: Vec::new(),
                files: Vec::new(),
                symlinks: Vec::new(),
                other: Vec::new(),
                errors: Vec::new(),
            };
            children
                .iter_mut()
                .for_each(|dir_entry_result| match dir_entry_result {
                    Ok(dir_entry) => {
                        let file_type = dir_entry.file_type;
                        let file_name = dir_entry.file_name.clone().into_string().unwrap();
                        if path.is_none() {
                            path = Some(dir_entry.parent_path.to_str().unwrap().to_string());
                        }
                        if file_type.is_symlink() {
                            toc.symlinks.push(file_name);
                        } else if file_type.is_dir() {
                            toc.dirs.push(file_name);
                        } else if file_type.is_file() {
                            toc.files.push(file_name);
                        } else {
                            toc.other.push(file_name);
                        }
                    }
                    Err(e) => toc.errors.push(e.to_string()),
                });
            if return_type == RETURN_TYPE_WALK {
                tx_clone
                    .send(WalkResult::WalkEntry(WalkEntry {
                        path: path.unwrap(),
                        toc: toc,
                    }))
                    .unwrap();
            } else {
                tx_clone
                    .send(WalkResult::WalkEntryExt(WalkEntryExt {
                        path: path.unwrap(),
                        toc: toc,
                    }))
                    .unwrap();
            }
        })
    {}
    match &duration {
        Some(d) => {
            let dt = start_time.elapsed().as_millis() as f64;
            d.store(dt.to_bits(), Ordering::Relaxed);
        }
        None => {}
    }
    BUSY.store(false, Ordering::Relaxed);
}

#[pyfunction]
pub fn toc(
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
) -> PyResult<Toc> {
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
    let toc = Arc::new(Mutex::new(Toc {
        dirs: Vec::new(),
        files: Vec::new(),
        symlinks: Vec::new(),
        other: Vec::new(),
        errors: Vec::new(),
    }));
    let toc_cloned = toc.clone();
    let rc: Result<(), Error> = py.allow_threads(|| {
        rs_toc(
            root_path,
            sorted.unwrap_or(false),
            skip_hidden.unwrap_or(false),
            max_depth.unwrap_or(::std::usize::MAX),
            filter,
            toc_cloned,
            None,
            None,
        );
        Ok(())
    });
    match rc {
        Err(e) => return Err(PyRuntimeError::new_err(e.to_string())),
        _ => (),
    }
    let toc_cloned = toc.lock().unwrap().clone();
    Ok(toc_cloned.into())
}

#[pyclass]
#[derive(Debug)]
pub struct Walk {
    // Options
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    max_depth: usize,
    return_type: u8,
    filter: Option<Filter>,
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
            self.filter.clone(),
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
        let root_path = self.root_path.clone();
        let sorted = self.sorted;
        let skip_hidden = self.skip_hidden;
        let max_depth = self.max_depth;
        let filter = self.filter.clone();
        let return_type = self.return_type;
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
                    filter,
                    toc,
                    Some(duration),
                    Some(alive),
                )
            }));
        } else {
            self.thr = Some(thread::spawn(move || {
                rs_walk_iter(
                    root_path,
                    sorted,
                    skip_hidden,
                    max_depth,
                    filter,
                    return_type,
                    Some(duration),
                    Some(alive),
                    tx.unwrap(),
                )
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
    fn new(
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
        Ok(Walk {
            root_path: root_path,
            sorted: sorted.unwrap_or(false),
            skip_hidden: skip_hidden.unwrap_or(false),
            max_depth: max_depth.unwrap_or(::std::usize::MAX),
            return_type: return_type.unwrap_or(RETURN_TYPE_WALK),
            filter: filter,
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
        match ty {
            Some(ty) => {
                if ty
                    .eq(Python::acquire_gil().python().get_type::<PyValueError>())
                    .unwrap()
                {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }

    #[getter]
    fn toc(&self) -> PyResult<Toc> {
        let mut toc_locked = self.toc.lock().unwrap();
        let tmp_toc = toc_locked.clone();
        toc_locked.deref_mut().clear();
        Ok(tmp_toc)
    }

    #[getter]
    fn duration(&self) -> f64 {
        f64::from_bits(self.duration.load(Ordering::Relaxed)) * 0.001
    }

    fn has_results(&self) -> bool {
        self.has_results
    }

    fn as_dict(&self) -> PyObject {
        let gil = Python::acquire_gil();
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
        pyresult.into()
    }

    fn collect(&mut self) -> PyResult<Toc> {
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        let gil = Python::acquire_gil();
        let rc: Result<(), Error> = gil.python().allow_threads(|| {
            self.rs_collect();
            Ok(())
        });
        self.alive = None;
        match rc {
            Err(e) => return Err(PyRuntimeError::new_err(e.to_string())),
            _ => (),
        }
        self.has_results = true;
        Ok(Arc::clone(&self.toc).lock().unwrap().clone())
    }

    fn start(&mut self) -> PyResult<bool> {
        if !self.rs_start(None) {
            return Err(PyRuntimeError::new_err("Thread already running"));
        }
        Ok(true)
    }

    fn stop(&mut self) -> PyResult<bool> {
        if !self.rs_stop() {
            return Err(PyRuntimeError::new_err("Thread not running"));
        }
        Ok(true)
    }

    fn busy(&self) -> bool {
        self.rs_busy()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<Walk>> {
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
        match &slf.rx {
            Some(rx) => match rx.recv() {
                Ok(val) => match val {
                    WalkResult::Toc(toc) => Ok(Some(toc.to_object(gil.python()))),
                    WalkResult::WalkEntry(entry) => Ok(Some(entry.to_object(gil.python()))),
                    WalkResult::WalkEntryExt(entry) => Ok(Some(entry.to_object(gil.python()))),
                },
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }
}

#[pymodule]
#[pyo3(name = "walk")]
pub fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Walk>()?;
    m.add_wrapped(wrap_pyfunction!(toc))?;
    m.add_wrapped(wrap_pyfunction!(ts_busy))?;
    m.add_wrapped(wrap_pyfunction!(ts_count))?;
    Ok(())
}
