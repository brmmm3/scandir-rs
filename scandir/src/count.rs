use std::collections::HashSet;
use std::fs::Metadata;
use std::io::Error;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use flume::{unbounded, Receiver, Sender};
use jwalk_meta::WalkDirGeneric;

use crate::common::{check_and_expand_path, create_filter, filter_children, get_root_path_len};
use crate::def::{Filter, Options, ReturnType};
use crate::Statistics;

fn count_thread(
    options: Options,
    filter: Option<Filter>,
    tx: Sender<Statistics>,
    stop: Arc<AtomicBool>,
) {
    let mut statistics = Statistics::new();

    let dir_entry: jwalk_meta::DirEntry<((), Option<Result<Metadata, Error>>)> =
        jwalk_meta::DirEntry::from_path(
            0,
            &options.root_path,
            true,
            true,
            options.follow_links,
            Arc::new(Vec::new()),
        )
        .unwrap();

    if !dir_entry.file_type.is_dir() {
        if dir_entry.file_type.is_symlink() {
            statistics.slinks += 1;
            statistics.usage += 4096;
            statistics.size += 4096;
        } else if dir_entry.file_type.is_file() {
            statistics.files += 1;
            if let Some(ref metadata) = dir_entry.metadata {
                let file_size = metadata.size;
                let mut blocks = file_size >> 12;
                if blocks << 12 < file_size {
                    blocks += 1;
                }
                statistics.usage += blocks << 12;
                statistics.size += file_size;
            }
        } else {
            #[cfg(unix)]
            if let Some(ref metadata) = dir_entry.metadata_ext {
                {
                    if metadata.st_rdev > 0 {
                        statistics.devices += 1;
                    } else if (metadata.st_mode & 4096) != 0 {
                        statistics.pipes += 1;
                    }
                }
            }
            statistics.usage += 4096;
            statistics.size += 4096;
        }
        statistics.duration = 0.01;
        let _ = tx.send(statistics);
        return;
    }

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
    let root_path_len = get_root_path_len(&options.root_path);
    let max_file_cnt = options.max_file_cnt as i32;
    for result in WalkDirGeneric::<((), Option<Result<Metadata, Error>>)>::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .sort(false)
        .max_depth(options.max_depth)
        .read_metadata(true)
        .read_metadata_ext(options.return_type == ReturnType::Ext)
        .process_read_dir(move |_, root_dir, _, children| {
            if let Some(root_dir) = root_dir.to_str() {
                if root_dir.len() + 1 < root_path_len {
                    return;
                }
            } else {
                return;
            }
            filter_children(children, &filter, root_path_len);
        })
    {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        match &result {
            Ok(v) => {
                if v.depth == 0 {
                    continue;
                }
                let file_type = v.file_type;
                if file_type.is_file() {
                    files += 1;
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
                                    files -= 1;
                                } else {
                                    file_indexes.insert(metadata.st_ino);
                                }
                            }
                        }
                        #[cfg(windows)]
                        {
                            if let Some(nlink) = metadata.number_of_links {
                                if nlink > 1 {
                                    if let Some(ino) = metadata.file_index {
                                        if file_indexes.contains(&ino) {
                                            hlinks += 1;
                                            files -= 1;
                                        } else {
                                            file_indexes.insert(ino);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if file_type.is_dir() {
                    dirs += 1;
                    usage += 4096;
                    size += 4096;
                } else if file_type.is_symlink() {
                    slinks += 1;
                    usage += 4096;
                    size += 4096;
                } else {
                    #[cfg(unix)]
                    if let Some(ref metadata) = v.metadata_ext {
                        {
                            if metadata.st_rdev > 0 {
                                devices += 1;
                            } else if (metadata.st_mode & 4096) != 0 {
                                pipes += 1;
                            }
                        }
                    }
                    usage += 4096;
                    size += 4096;
                }
                cnt += 1;
                if cnt >= 1000 || update_time.elapsed().as_millis() >= 10 {
                    statistics.dirs = dirs;
                    statistics.files = files;
                    statistics.slinks = slinks;
                    statistics.hlinks = hlinks;
                    statistics.size = size;
                    statistics.usage = usage;
                    statistics.duration = start_time.elapsed().as_secs_f64();
                    #[cfg(unix)]
                    {
                        statistics.devices = devices;
                        statistics.pipes = pipes;
                    }
                    let _ = tx.send(statistics.clone());
                    cnt = 0;
                    update_time = Instant::now();
                }
                if !file_type.is_dir() && max_file_cnt > 0 && files > max_file_cnt {
                    break;
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
    statistics.duration = start_time.elapsed().as_secs_f64();
    #[cfg(unix)]
    {
        statistics.devices = devices;
        statistics.pipes = pipes;
    }
    let _ = tx.send(statistics);
}

#[derive(Debug)]
pub struct Count {
    // Options
    options: Options,
    // Results
    pub statistics: Statistics,
    duration: Arc<Mutex<f64>>,
    finished: Arc<AtomicBool>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<Statistics>>,
}

impl Count {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Result<Self, Error> {
        Ok(Count {
            options: Options {
                root_path: check_and_expand_path(root_path)?,
                sorted: false,
                skip_hidden: false,
                max_depth: usize::MAX,
                max_file_cnt: usize::MAX,
                dir_include: None,
                dir_exclude: None,
                file_include: None,
                file_exclude: None,
                case_sensitive: false,
                follow_links: false,
                return_type: ReturnType::Base,
            },
            statistics: Statistics::new(),
            duration: Arc::new(Mutex::new(0.0)),
            finished: Arc::new(AtomicBool::new(false)),
            thr: None,
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
            0 => usize::MAX,
            _ => depth,
        };
        self
    }

    /// Set maximum number of files to collect
    pub fn max_file_cnt(mut self, max_file_cnt: usize) -> Self {
        self.options.max_file_cnt = match max_file_cnt {
            0 => usize::MAX,
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

    /// Set follow symlinks
    pub fn follow_links(mut self, follow_links: bool) -> Self {
        self.options.follow_links = follow_links;
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

    /// Same as method `extended`, but without moving the instance
    pub fn set_extended(&mut self, extended: bool) {
        self.options.return_type = match extended {
            false => ReturnType::Base,
            true => ReturnType::Ext,
        };
    }

    pub fn clear(&mut self) {
        self.statistics.clear();
        *self.duration.lock().unwrap() = 0.0;
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.busy() {
            return Err(Error::other("Busy"));
        }
        self.clear();
        let options = self.options.clone();
        let filter = create_filter(&options)?;
        let (tx, rx) = unbounded();
        self.rx = Some(rx);
        self.stop.store(false, Ordering::Relaxed);
        let stop = self.stop.clone();
        let duration = self.duration.clone();
        let finished = self.finished.clone();
        self.thr = Some(thread::spawn(move || {
            let start_time = Instant::now();
            count_thread(options, filter, tx, stop);
            *duration.lock().unwrap() = start_time.elapsed().as_secs_f64();
            finished.store(true, Ordering::Relaxed);
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
            while let Ok(s) = rx.try_recv() {
                self.statistics = s;
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
        self.finished.load(Ordering::Relaxed)
    }

    pub fn busy(&self) -> bool {
        if let Some(ref thr) = self.thr {
            !thr.is_finished()
        } else {
            false
        }
    }

    // For debugging

    pub fn options(&self) -> Options {
        self.options.clone()
    }
}
