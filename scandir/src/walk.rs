use std::fmt::Debug;
use std::fs;
use std::io::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use flume::{unbounded, Receiver, Sender};
use jwalk::WalkDirGeneric;

use crate::common::check_and_expand_path;
use crate::common::{create_filter, filter_children};
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
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: i32,
    max_file_cnt: i32,
    filter: Option<Filter>,
    tx: Sender<(String, Toc)>,
    stop: Arc<AtomicBool>,
) {
    if max_depth == 0 {
        max_depth = std::i32::MAX;
    }
    let root_path_len = root_path.to_string_lossy().len();
    let file_cnt = Arc::new(AtomicI32::new(0));
    let file_cnt_cloned = file_cnt.clone();
    let tx_cloned = tx.clone();
    for _ in WalkDirGeneric::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth as usize)
        .process_read_dir(move |_, root_dir, _, children| {
            let root_dir = root_dir.to_str().unwrap();
            if root_dir.len() < root_path_len {
                return;
            }
            filter_children(children, &filter, root_path_len + 1);
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
                    let _ = tx_cloned.send((root_dir[root_path_len + 1..].to_owned(), toc));
                } else {
                    let _ = tx_cloned.send(("".to_owned(), toc));
                }
            }
            let file_cnt_new = file_cnt_cloned.load(Ordering::Relaxed) + children.len() as i32;
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
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    max_depth: i32,
    max_file_cnt: i32,
    filter: Option<Filter>,
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
    pub fn new(
        root_path: &str,
        sorted: bool,
        skip_hidden: bool,
        max_depth: i32,
        max_file_cnt: i32,
        dir_include: Option<Vec<String>>,
        dir_exclude: Option<Vec<String>>,
        file_include: Option<Vec<String>>,
        file_exclude: Option<Vec<String>>,
        case_sensitive: bool,
    ) -> Result<Self, Error> {
        let root_path = check_and_expand_path(&root_path)?;
        let filter = create_filter(
            dir_include,
            dir_exclude,
            file_include,
            file_exclude,
            case_sensitive,
        )?;
        Ok(Walk {
            root_path,
            sorted,
            skip_hidden,
            max_depth,
            max_file_cnt,
            filter,
            entries: Vec::new(),
            duration: Arc::new(Mutex::new(0.0)),
            has_errors: false,
            thr: None,
            alive: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            rx: None,
        })
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.has_errors = false;
        *self.duration.lock().unwrap() = 0.0;
    }

    pub fn start(&mut self) -> bool {
        if self.thr.is_some() {
            return false;
        }
        self.clear();
        let root_path = self.root_path.clone();
        let sorted = self.sorted;
        let skip_hidden = self.skip_hidden;
        let max_depth = self.max_depth;
        let max_file_cnt = self.max_file_cnt;
        let filter = self.filter.clone();
        let (tx, rx) = unbounded();
        self.rx = Some(rx);
        self.alive.store(true, Ordering::Relaxed);
        self.stop.store(false, Ordering::Relaxed);
        let alive = self.alive.clone();
        let stop = self.stop.clone();
        let duration = self.duration.clone();
        self.thr = Some(thread::spawn(move || {
            let start_time = Instant::now();
            toc_thread(
                root_path,
                sorted,
                skip_hidden,
                max_depth,
                max_file_cnt,
                filter,
                tx,
                stop,
            );
            alive.store(false, Ordering::Relaxed);
            *duration.lock().unwrap() = start_time.elapsed().as_millis() as f64 * 0.001;
        }));
        true
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

    pub fn collect(&mut self) -> Toc {
        if !self.finished() {
            if !self.busy() {
                self.start();
            }
            self.join();
        }
        let mut toc = Toc::new();
        for (root_dir, dir_toc) in self.results(true) {
            toc.extend(&root_dir, &dir_toc);
        }
        toc
    }

    pub fn results(&mut self, return_all: bool) -> Vec<(String, Toc)> {
        let entries = self.receive_all();
        self.entries.extend_from_slice(&entries);
        if return_all {
            return self.entries.clone();
        }
        entries
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&mut self) -> bool {
        *self.duration.lock().unwrap() > 0.0
    }

    pub fn has_errors(&mut self) -> bool {
        !self.has_errors
    }

    pub fn busy(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }
}
