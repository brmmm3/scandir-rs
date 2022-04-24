use std::collections::HashSet;
use std::fs;
use std::fs::Metadata;
use std::io::Error;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::prelude::*;
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

#[derive(Debug, Clone)]
pub struct Statistics {
    pub dirs: i32,
    pub files: i32,
    pub slinks: i32,
    pub hlinks: i32,
    pub devices: i32,
    pub pipes: i32,
    pub size: u64,
    pub usage: u64,
    pub errors: Vec<String>,
    pub duration: f64,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
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
        }
    }

    pub fn clear(&mut self) {
        self.dirs = 0;
        self.files = 0;
        self.slinks = 0;
        self.hlinks = 0;
        self.devices = 0;
        self.pipes = 0;
        self.size = 0;
        self.usage = 0;
        self.errors.clear();
        self.duration = 0.0;
    }
}

fn count_thread(
    root_path: PathBuf,
    extended: bool, // If true: Count also hardlinks, devices, pipes, size and usage
    skip_hidden: bool,
    mut max_depth: i32,
    max_file_cnt: i32,
    filter: Option<Filter>,
    tx: Sender<Statistics>,
    stop: Arc<AtomicBool>,
) {
    let mut dirs: i32 = 0;
    let mut files: i32 = 0;
    let mut slinks: i32 = 0;
    let mut hlinks: i32 = 0;
    let mut size: u64 = 0;
    let mut usage: u64 = 0;
    #[cfg(unix)]
    let mut devices: i32 = 0;
    #[cfg(unix)]
    let mut pipes: i32 = 0;
    let mut cnt: i32 = 0;
    let start_time = Instant::now();
    let mut update_time = start_time;
    let mut file_indexes: HashSet<u64> = HashSet::new();
    let mut statistics = Statistics::new();
    if max_depth == 0 {
        max_depth = std::i32::MAX;
    }
    let root_path_len = root_path.to_string_lossy().len() + 1;
    let file_cnt = Arc::new(AtomicI32::new(0));
    let stop_cloned = stop.clone();
    let file_cnt_cloned = file_cnt.clone();
    let tx_cloned = tx.clone();
    for entry in WalkDirGeneric::<((), Option<Result<Metadata, Error>>)>::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(false)
        .max_depth(max_depth as usize)
        .process_read_dir(move |_, _, _, children| {
            if stop_cloned.load(Ordering::Relaxed) {
                return;
            }
            filter_children(children, &filter, root_path_len);
            if children.is_empty() {
                return;
            }
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    if extended {
                        dir_entry.client_state = Some(fs::metadata(dir_entry.path()));
                    }
                }
            });
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
                    statistics.dirs = dirs;
                    statistics.files = files;
                    statistics.slinks = slinks;
                    statistics.hlinks = hlinks;
                    statistics.size = size;
                    statistics.usage = usage;
                    statistics.duration = start_time.elapsed().as_millis() as f64 * 0.001;
                    #[cfg(unix)]
                    {
                        statistics.devices = devices;
                        statistics.pipes = pipes;
                    }
                    let _ = tx_cloned.send(statistics.clone());
                    cnt = 0;
                    update_time = Instant::now();
                }
            }
            Err(e) => statistics.errors.push(e.to_string()), // TODO: Need to fetch failed path from somewhere
        }
    }
    statistics.dirs = dirs;
    statistics.files = files;
    statistics.slinks = slinks;
    statistics.hlinks = hlinks;
    statistics.size = size;
    statistics.usage = usage;
    statistics.duration = start_time.elapsed().as_millis() as f64 * 0.001;
    #[cfg(unix)]
    {
        statistics.devices = devices;
        statistics.pipes = pipes;
    }
    let _ = tx_cloned.send(statistics);
}

#[derive(Debug)]
pub struct Count {
    // Options
    root_path: PathBuf,
    extended: bool,
    skip_hidden: bool,
    max_depth: i32,
    max_file_cnt: i32,
    filter: Option<Filter>,
    // Results
    statistics: Statistics,
    duration: Arc<Mutex<f64>>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<Statistics>>,
}

impl Count {
    pub fn new(
        root_path: &str,
        extended: bool,
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
        Ok(Count {
            root_path,
            extended,
            skip_hidden,
            max_depth,
            max_file_cnt,
            filter,
            statistics: Statistics::new(),
            duration: Arc::new(Mutex::new(0.0)),
            thr: None,
            alive: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            rx: None,
        })
    }

    pub fn clear(&mut self) {
        self.statistics.clear();
        *self.duration.lock().unwrap() = 0.0;
    }

    pub fn start(&mut self) -> bool {
        if self.thr.is_some() {
            return false;
        }
        self.clear();
        let root_path = self.root_path.clone();
        let extended = self.extended;
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
            count_thread(
                root_path,
                extended,
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

    fn receive_all(&mut self) -> Statistics {
        if let Some(ref rx) = self.rx {
            loop {
                match rx.try_recv() {
                    Ok(s) => self.statistics = s,
                    Err(_) => break,
                }
            }
        }
        self.statistics.clone()
    }

    pub fn collect(&mut self) -> Statistics {
        if !self.finished() {
            if !self.busy() {
                self.start();
            }
            self.join();
        }
        self.receive_all()
    }

    pub fn results(&mut self) -> Statistics {
        self.receive_all()
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&self) -> bool {
        self.statistics.duration > 0.0
    }

    pub fn has_errors(&mut self) -> bool {
        !self.statistics.errors.is_empty()
    }

    pub fn busy(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }
}
