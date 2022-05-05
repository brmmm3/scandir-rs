use std::collections::HashSet;
use std::fs::Metadata;
use std::io::{Error, ErrorKind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use flume::{unbounded, Receiver, Sender};
use jwalk::WalkDirGeneric;

use crate::common::{check_and_expand_path, create_filter, filter_children, get_root_path_len};
use crate::def::{Filter, Options, ReturnType};

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
    options: Options,
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
    let root_path_len = get_root_path_len(&options.root_path);
    let max_file_cnt = options.max_file_cnt;
    let file_cnt = Arc::new(AtomicUsize::new(0));
    let file_cnt_cloned = file_cnt.clone();
    let stop_cloned = stop.clone();
    let tx_cloned = tx.clone();
    for entry in WalkDirGeneric::<((), Option<Result<Metadata, Error>>)>::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .sort(false)
        .max_depth(options.max_depth)
        .read_metadata(true)
        .read_metadata_ext(options.return_type == ReturnType::Ext)
        .process_read_dir(move |_, root_dir, _, children| {
            if stop_cloned.load(Ordering::Relaxed) {
                return;
            }
            if let Some(root_dir) = root_dir.to_str() {
                if root_dir.len() + 1 < root_path_len {
                    return;
                }
            } else {
                return;
            }
            filter_children(children, &filter, root_path_len);
            if children.is_empty() {
                return;
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
        match &entry {
            Ok(v) => {
                if v.depth == 0 {
                    continue;
                }
                let file_type = v.file_type;
                if file_type.is_dir() {
                    dirs += 1;
                } else if file_type.is_file() {
                    files += 1;
                } else if file_type.is_symlink() {
                    slinks += 1;
                }
                if let Some(ref metadata) = v.metadata {
                    let file_size = metadata.size;
                    let mut blocks = file_size >> 12;
                    if blocks << 12 < file_size {
                        blocks += 1;
                    }
                    usage += blocks << 12;
                    size += file_size;
                }
                if let Some(ref metadata) = v.metadata_ext {
                    #[cfg(unix)]
                    {
                        if metadata.st_nlink > 1 {
                            if file_indexes.contains(&metadata.st_ino) {
                                hlinks += 1;
                            } else {
                                file_indexes.insert(metadata.st_ino);
                            }
                        }
                        if metadata.st_rdev > 0 {
                            devices += 1;
                        }
                        if (metadata.st_mode & 4096) != 0 {
                            pipes += 1;
                        }
                    }
                    #[cfg(windows)]
                    {
                        if let Some(nlink) = metadata.number_of_links {
                            if nlink > 1 {
                                if let Some(ino) = metadata.file_index {
                                    if file_indexes.contains(&ino) {
                                        hlinks += 1;
                                    } else {
                                        file_indexes.insert(ino);
                                    }
                                }
                            }
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
    options: Options,
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
    pub fn new(root_path: &str) -> Result<Self, Error> {
        Ok(Count {
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
                return_type: ReturnType::Base,
            },
            statistics: Statistics::new(),
            duration: Arc::new(Mutex::new(0.0)),
            thr: None,
            alive: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            rx: None,
        })
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
    pub fn extended(mut self, extended: bool) -> Self {
        self.options.return_type = match extended {
            true => ReturnType::Ext,
            false => ReturnType::Base,
        };
        self
    }

    pub fn clear(&mut self) {
        self.statistics.clear();
        *self.duration.lock().unwrap() = 0.0;
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.busy() {
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
            count_thread(options, filter, tx, stop);
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

    pub fn collect(&mut self) -> Result<Statistics, Error> {
        if !self.finished() {
            if !self.busy() {
                self.start()?;
            }
            self.join();
        }
        Ok(self.receive_all())
    }

    pub fn has_results(&self) -> bool {
        if let Some(ref rx) = self.rx {
            if !rx.is_empty() {
                return true;
            }
        }
        false
    }

    pub fn results(&mut self) -> Statistics {
        self.receive_all()
    }

    pub fn has_errors(&mut self) -> bool {
        !self.statistics.errors.is_empty()
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&self) -> bool {
        self.statistics.duration > 0.0
    }

    pub fn busy(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    // For debugging

    pub fn options(&self) -> Options {
        self.options.clone()
    }
}
