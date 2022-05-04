use std::fmt::Debug;
use std::fs;
use std::io::{Error, ErrorKind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use flume::{unbounded, Receiver, Sender};
use jwalk::WalkDirGeneric;

use crate::common::{check_and_expand_path, create_filter, filter_children, get_root_path_len};
use crate::def::*;

#[inline]
fn update_toc(
    dir_entry: &jwalk::DirEntry<((), Option<Result<fs::Metadata, Error>>)>,
    toc: &mut Toc,
) {
    let file_type = dir_entry.file_type;
    let key = dir_entry.file_name.clone().into_string().unwrap();
    if file_type.is_symlink() {
        toc.symlinks.push(key);
    } else if file_type.is_dir() {
        toc.dirs.push(key);
    } else if file_type.is_file() {
        toc.files.push(key);
    } else {
        toc.other.push(key);
    }
}

pub fn toc_thread(
    options: Options,
    filter: Option<Filter>,
    tx: Sender<(String, Toc)>,
    stop: Arc<AtomicBool>,
) {
    let root_path_len = get_root_path_len(&options.root_path);
    let max_file_cnt = options.max_file_cnt;
    let file_cnt = Arc::new(AtomicUsize::new(0));
    let file_cnt_cloned = file_cnt.clone();
    let stop_cloned = stop.clone();
    let tx_cloned = tx.clone();
    for _ in WalkDirGeneric::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .sort(options.sorted)
        .max_depth(options.max_depth)
        .process_read_dir(move |_, root_dir, _, children| {
            if stop_cloned.load(Ordering::Relaxed) {
                return;
            }
            let root_dir = root_dir.to_str();
            if root_dir.is_none() {
                return;
            }
            let root_dir = root_dir.unwrap();
            if root_dir.len() + 1 < root_path_len {
                return;
            }
            filter_children(children, &filter, root_path_len);
            if children.is_empty() {
                return;
            }
            let mut toc = Toc::new();
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    update_toc(&dir_entry, &mut toc);
                }
            });
            if !toc.is_empty() {
                if root_dir.len() > root_path_len {
                    let _ = tx_cloned.send((root_dir[root_path_len..].to_owned(), toc));
                } else {
                    let _ = tx_cloned.send(("".to_owned(), toc));
                }
            }
            let file_cnt_new = file_cnt_cloned.load(Ordering::Relaxed) + children.len();
            file_cnt_cloned.store(file_cnt_new, Ordering::Relaxed);
        })
    {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        if max_file_cnt > 0 && file_cnt.load(Ordering::Relaxed) > max_file_cnt {
            break;
        }
    }
}

#[derive(Debug)]
pub struct Walk {
    // Options
    options: Options,
    // Results
    entries: Vec<(String, Toc)>,
    duration: Arc<Mutex<f64>>,
    has_errors: bool,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<(String, Toc)>>,
}

impl Walk {
    pub fn new(root_path: &str) -> Result<Self, Error> {
        Ok(Walk {
            options: Options {
                root_path: check_and_expand_path(&root_path)?,
                sorted: false,
                skip_hidden: true,
                max_depth: std::usize::MAX,
                max_file_cnt: std::usize::MAX,
                dir_include: None,
                dir_exclude: None,
                file_include: None,
                file_exclude: None,
                case_sensitive: false,
                return_type: ReturnType::Fast,
            },
            entries: Vec::new(),
            duration: Arc::new(Mutex::new(0.0)),
            has_errors: false,
            thr: None,
            alive: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            rx: None,
        })
    }

    /// Return results in sorted order.
    pub fn sorted(mut self, sorted: bool) -> Self {
        self.options.sorted = sorted;
        self
    }

    /// Skip hidden entries. Enabled by default.
    pub fn skip_hidden(mut self, skip_hidden: bool) -> Self {
        self.options.skip_hidden = skip_hidden;
        self
    }

    /// Set the maximum depth of entries yield by the iterator.
    ///
    /// The smallest depth is `0` and always corresponds to the path given
    /// to the `new` function on this type. Its direct descendents have depth
    /// `1`, and their descendents have depth `2`, and so on.
    ///
    /// Note that this will not simply filter the entries of the iterator, but
    /// it will actually avoid descending into directories when the depth is
    /// exceeded.
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.options.max_depth = match depth {
            0 => std::usize::MAX,
            _ => depth,
        };
        self
    }

    /// Set maximum number of files to collect
    pub fn max_file_cnt(mut self, max_file_cnt: usize) -> Self {
        self.options.max_file_cnt = match max_file_cnt {
            0 => std::usize::MAX,
            _ => max_file_cnt,
        };
        self
    }

    /// Set directory include filter
    pub fn dir_include(mut self, dir_include: Option<Vec<String>>) -> Self {
        self.options.dir_include = dir_include;
        self
    }

    /// Set directory exclude filter
    pub fn dir_exclude(mut self, dir_exclude: Option<Vec<String>>) -> Self {
        self.options.dir_exclude = dir_exclude;
        self
    }

    /// Set file include filter
    pub fn file_include(mut self, file_include: Option<Vec<String>>) -> Self {
        self.options.file_include = file_include;
        self
    }

    /// Set file exclude filter
    pub fn file_exclude(mut self, file_exclude: Option<Vec<String>>) -> Self {
        self.options.file_exclude = file_exclude;
        self
    }

    /// Set case sensitive filename filtering
    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.options.case_sensitive = case_sensitive;
        self
    }

    /// Set extended file type counting
    pub fn return_type(mut self, return_type: ReturnType) -> Self {
        self.options.return_type = return_type;
        self
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.has_errors = false;
        *self.duration.lock().unwrap() = 0.0;
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.thr.is_some() {
            return Err(Error::new(ErrorKind::Other, "Busy"));
        }
        self.clear();
        let options = self.options.clone();
        let filter = create_filter(&options)?;
        let (tx, rx) = unbounded();
        self.rx = Some(rx);
        self.alive.store(true, Ordering::Relaxed);
        self.stop.store(false, Ordering::Relaxed);
        let alive = self.alive.clone();
        let stop = self.stop.clone();
        let duration = self.duration.clone();
        self.thr = Some(thread::spawn(move || {
            let start_time = Instant::now();
            toc_thread(options, filter, tx, stop);
            alive.store(false, Ordering::Relaxed);
            *duration.lock().unwrap() = start_time.elapsed().as_millis() as f64 * 0.001;
        }));
        Ok(())
    }

    pub fn join(&mut self) -> bool {
        if let Some(thr) = self.thr.take() {
            if let Err(_e) = thr.join() {
                return false;
            }
            return true;
        }
        false
    }

    pub fn stop(&mut self) -> bool {
        if let Some(thr) = self.thr.take() {
            self.stop.store(true, Ordering::Relaxed);
            if let Err(_e) = thr.join() {
                return false;
            }
            return true;
        }
        false
    }

    fn receive_all(&mut self) -> Vec<(String, Toc)> {
        let mut entries = Vec::new();
        if let Some(ref rx) = self.rx {
            loop {
                match rx.try_recv() {
                    Ok(entry) => {
                        if !entry.1.errors.is_empty() {
                            self.has_errors = true;
                        }
                        entries.push(entry);
                    }
                    Err(_) => break,
                }
            }
        }
        entries
    }

    pub fn collect(&mut self) -> Result<Toc, Error> {
        if !self.finished() {
            if !self.busy() {
                self.start()?;
            }
            self.join();
        }
        let mut toc = Toc::new();
        for (root_dir, dir_toc) in self.results(true) {
            toc.extend(&root_dir, &dir_toc);
        }
        Ok(toc)
    }

    pub fn has_results(&mut self) -> bool {
        if let Some(ref rx) = self.rx {
            if !rx.is_empty() {
                return true;
            }
        }
        !self.entries.is_empty()
    }

    pub fn results_cnt(&mut self, update: bool) -> usize {
        if update {
            self.results(false);
        }
        self.entries.len()
    }

    pub fn results(&mut self, return_all: bool) -> Vec<(String, Toc)> {
        let entries = self.receive_all();
        self.entries.extend_from_slice(&entries);
        if return_all {
            return self.entries.clone();
        }
        entries
    }

    pub fn has_errors(&mut self) -> bool {
        !self.has_errors
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&mut self) -> bool {
        *self.duration.lock().unwrap() > 0.0
    }

    pub fn busy(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    // For debugging

    pub fn options(&self) -> Options {
        self.options.clone()
    }
}
