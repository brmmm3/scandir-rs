use std::iter;
use std::time::UNIX_EPOCH;
use std::collections::HashMap;
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

fn walk(
    root_path: String,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
    result: Arc<Mutex<Entries>>,
    alive: Option<Arc<AtomicBool>>,
) {
    #[cfg(unix)]
    let root_path = expanduser(root_path).unwrap();
    let mut entries = HashMap::new();
    let mut errors = Vec::new();
    let mut cnt = 0;
    for entry in WalkDir::new(root_path)
        .skip_hidden(skip_hidden.unwrap_or(false))
        .sort(sorted.unwrap_or(false))
        .preload_metadata(metadata.unwrap_or(false))
        .preload_metadata_ext(metadata_ext.unwrap_or(false))
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
                entries.insert(key.to_str().unwrap().to_string(), entry);
            }
            Err(e) => errors.push(e.to_string())  // TODO: Need to fetch failed path from somewhere
        };
        cnt += 1;
        if cnt >= 1000 {
            let mut results = result.lock().unwrap();
            if !entries.is_empty() {
                for (k, v) in entries.drain() {
                    results.entries.insert(k, v);
                }            }
            if results.errors.len() < errors.len() {
                results.errors.extend_from_slice(&errors);
                errors.clear();
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
}

#[pyfunction]
pub fn entries(
    py: Python,
    root_path: String,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
) -> PyResult<PyObject> {
    let result = Arc::new(Mutex::new(Entries {
        entries: HashMap::new(),
        errors: Vec::new(),
    }));
    let result_clone = result.clone();
    let rc: std::result::Result<(), std::io::Error> = py.allow_threads(|| {
        walk(root_path, sorted, skip_hidden, metadata, metadata_ext, result_clone, None);
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
        let mut results = result.lock().unwrap();
        for (k, v) in results.entries.drain() {
            pyresult.set_item(k, PyRef::new(py, v).unwrap())?;
        }
        if !results.errors.is_empty() {
            pyresult.set_item(py.None(), results.errors.to_vec())?;
        }
    }
    Ok(pyresult.into())
}

#[pyclass]
#[derive(Debug)]
pub struct Scandir {
    // Options
    root_path: String,
    sorted: Option<bool>,
    skip_hidden: Option<bool>,
    metadata: Option<bool>,
    metadata_ext: Option<bool>,
    // Results
    entries: Arc<Mutex<Entries>>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Option<Weak<AtomicBool>>,
    exit_called: bool,
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
    ) {
        obj.init(Scandir {
            root_path: String::from(root_path),
            sorted: sorted,
            skip_hidden: skip_hidden,
            metadata: metadata,
            metadata_ext: metadata_ext,
            entries: Arc::new(Mutex::new(Entries { 
                entries: HashMap::new(),
                errors: Vec::new(),
            })),
            thr: None,
            alive: None,
            exit_called: false,
        });
    }

    #[getter]
    fn entries(&self) -> PyResult<Entries> {
       Ok(Arc::clone(&self.entries).lock().unwrap().clone())
    }

    fn collect(&self) -> PyResult<Entries> {
        walk(self.root_path.clone(),
             self.sorted, self.skip_hidden, self.metadata, self.metadata_ext,
             self.entries.clone(), None);
        Ok(Arc::clone(&self.entries).lock().unwrap().clone())
    }

    fn start(&mut self) -> PyResult<bool> {
        if self.thr.is_some() {
            return Err(exceptions::RuntimeError::py_err("Thread already running"))
        }
        let root_path = String::from(&self.root_path);
        let sorted = self.sorted;
        let skip_hidden = self.skip_hidden;
        let metadata = self.metadata;
        let metadata_ext = self.metadata_ext;
        let entries = self.entries.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            walk(root_path, sorted, skip_hidden, metadata, metadata_ext, entries, Some(alive))
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
impl pyo3::class::PyObjectProtocol for Scandir {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for Scandir {
    fn __enter__(&'p mut self) -> PyResult<()> {
        let root_path = String::from(&self.root_path);
        let sorted = self.sorted;
        let skip_hidden = self.skip_hidden;
        let metadata = self.metadata;
        let metadata_ext = self.metadata_ext;
        let entries = self.entries.clone();
        let alive = Arc::new(AtomicBool::new(true));
        self.alive = Some(Arc::downgrade(&alive));
        self.thr = Some(thread::spawn(move || {
            walk(root_path, sorted, skip_hidden, metadata, metadata_ext,
                 entries, Some(alive))
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

#[pyclass]
pub struct ScandirIter {
    iter: Box<dyn iter::Iterator<Item = i32> + Send>,
}

#[pymethods]
impl ScandirIter {
    #[new]
    fn __new__(
        obj: &PyRawObject,
    ) {
        obj.init(ScandirIter { iter: Box::new(5..8),
        });
    }
}

#[pyproto]
impl<'p> PyIterProtocol for ScandirIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<ScandirIter>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<i32>> {
        Ok(slf.iter.next())
    }
}

#[pymodule(scandir)]
fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Scandir>()?;
    m.add_wrapped(wrap_pyfunction!(entries))?;
    Ok(())
}
