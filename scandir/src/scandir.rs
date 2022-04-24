use std::fs::{self, Metadata};
use std::io::{Error, ErrorKind};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, UNIX_EPOCH};

use flume::{unbounded, Receiver, Sender};

use jwalk::WalkDirGeneric;

use crate::common::check_and_expand_path;
use crate::common::{create_filter, filter_children};
use crate::def::*;

#[derive(Debug, Clone)]
pub enum Stats {
    ScandirResult(ScandirResult),
    Error(String),
    Duration(f64),
}

/// Scandir result
#[derive(Debug, Clone)]
pub struct Entry {
    /// Absolute file path
    pub path: String,
    /// File stats
    pub entry: Stats,
}

#[inline]
fn create_entry(
    root_path_len: usize,
    return_type: &ReturnType,
    dir_entry: &jwalk::DirEntry<((), Option<Result<Metadata, Error>>)>,
) -> (bool, Entry) {
    let file_type = dir_entry.file_type;
    let mut st_ctime: f64 = 0.0;
    let mut st_mtime: f64 = 0.0;
    let mut st_atime: f64 = 0.0;
    let mut st_mode: u32 = 0;
    let mut st_ino: u64 = 0;
    let mut st_dev: u64 = 0;
    let mut st_nlink: u64 = 0;
    let mut st_size: u64 = 0;
    let mut st_blksize: u64 = 4096;
    let mut st_blocks: u64 = 0;
    #[cfg(unix)]
    let mut st_uid: u32 = 0;
    #[cfg(windows)]
    let st_uid: u32 = 0;
    #[cfg(unix)]
    let mut st_gid: u32 = 0;
    #[cfg(windows)]
    let st_gid: u32 = 0;
    #[cfg(unix)]
    let mut st_rdev: u64 = 0;
    #[cfg(windows)]
    let st_rdev: u64 = 0;
    if let Ok(metadata) = fs::metadata(dir_entry.path()) {
        let duration = metadata
            .created()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap();
        st_ctime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let duration = metadata
            .modified()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap();
        st_mtime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let duration = metadata
            .accessed()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap();
        st_atime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        if return_type > &ReturnType::Base {
            #[cfg(unix)]
            {
                st_mode = metadata.mode();
                st_ino = metadata.ino();
                st_dev = metadata.dev() as u64;
                st_nlink = metadata.nlink() as u64;
                st_size = metadata.size();
                st_blksize = metadata.blksize();
                st_blocks = metadata.blocks();
                st_uid = metadata.uid();
                st_gid = metadata.gid();
                st_rdev = metadata.rdev();
            }
            #[cfg(windows)]
            {
                st_mode = metadata.file_attributes();
                if let Some(ino) = metadata.file_index() {
                    st_ino = ino;
                }
                if let Some(dev) = metadata.volume_serial_number() {
                    st_dev = dev as u64;
                }
                if let Some(nlink) = metadata.number_of_links() {
                    st_nlink = nlink as u64;
                }
                st_size = metadata.file_size();
                st_blksize = 4096;
                st_blocks = st_size >> 12;
                if st_blocks << 12 < st_size {
                    st_blocks += 1;
                }
            }
        }
    }
    let mut key = dir_entry.parent_path.to_path_buf();
    let file_name = dir_entry.file_name.clone().into_string().unwrap();
    key.push(&file_name);
    let key = key.to_str().unwrap().to_string();
    let path = key.get(root_path_len..).unwrap_or(&file_name).to_string();
    let is_file = file_type.is_file();
    let entry: ScandirResult = match return_type {
        ReturnType::Fast | ReturnType::Base => ScandirResult::DirEntry(DirEntry {
            path,
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
            st_ctime: st_ctime,
            st_mtime: st_mtime,
            st_atime: st_atime,
        }),
        ReturnType::Ext => ScandirResult::DirEntryExt(DirEntryExt {
            path,
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
            st_ctime: st_ctime,
            st_mtime: st_mtime,
            st_atime: st_atime,
            st_mode: st_mode,
            st_ino: st_ino,
            st_dev: st_dev,
            st_nlink: st_nlink,
            st_size: st_size,
            st_blksize: st_blksize,
            st_blocks: st_blocks,
            st_uid: st_uid,
            st_gid: st_gid,
            st_rdev: st_rdev,
        }),
        _ => ScandirResult::Error((path, "Wrong return type!".to_string())),
    };
    (
        is_file,
        Entry {
            path: key, // Absolute file path
            entry: Stats::ScandirResult(entry),
        },
    )
}

fn entries_thread(
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: i32,
    max_file_cnt: i32,
    filter: Option<Filter>,
    return_type: ReturnType,
    tx: Sender<Entry>,
    stop: Arc<AtomicBool>,
) {
    if max_depth == 0 {
        max_depth = std::i32::MAX;
    };
    let root_path_len = root_path.to_string_lossy().len() + 1;
    let file_cnt = Arc::new(AtomicI32::new(0));
    let stop_cloned = stop.clone();
    let file_cnt_cloned = file_cnt.clone();
    let tx_cloned = tx.clone();
    for _ in WalkDirGeneric::new(&root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .max_depth(max_depth as usize)
        .process_read_dir(move |_, _, _, children| {
            if stop_cloned.load(Ordering::Relaxed) {
                return;
            }
            filter_children(children, &filter, root_path_len);
            if children.is_empty() {
                return;
            }
            let mut local_file_cnt: i32 = 0;
            children.iter_mut().for_each(|dir_entry_result| {
                if stop_cloned.load(Ordering::Relaxed) {
                    return;
                }
                if let Ok(dir_entry) = dir_entry_result {
                    let (is_file, entry) = create_entry(root_path_len, &return_type, dir_entry);
                    if tx_cloned.send(entry).is_err() {
                        return;
                    }
                    if is_file {
                        local_file_cnt += 1;
                    }
                }
            });
            if local_file_cnt > 0 {
                file_cnt_cloned.store(
                    file_cnt_cloned.load(Ordering::Relaxed) + local_file_cnt,
                    Ordering::Relaxed,
                );
            }
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

/// Class for iterating a file tree and returning `Entry` objects
#[derive(Debug)]
pub struct Scandir {
    // Options
    root_path: PathBuf,
    sorted: bool,
    skip_hidden: bool,
    max_depth: i32,
    max_file_cnt: i32,
    return_type: ReturnType,
    filter: Option<Filter>,
    // Results
    entries: Vec<ScandirResult>,
    errors: Vec<(String, String)>,
    duration: Arc<Mutex<f64>>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    alive: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<Entry>>,
}

impl Scandir {
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
        return_type: ReturnType,
    ) -> Result<Self, Error> {
        if return_type > ReturnType::Ext {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Parameter return_type has invalid value",
            ));
        }
        let root_path = check_and_expand_path(&root_path)?;
        let filter = create_filter(
            dir_include,
            dir_exclude,
            file_include,
            file_exclude,
            case_sensitive,
        )?;
        Ok(Scandir {
            root_path,
            sorted,
            skip_hidden,
            max_depth,
            max_file_cnt,
            return_type,
            filter,
            entries: Vec::new(),
            errors: Vec::new(),
            duration: Arc::new(Mutex::new(0.0)),
            thr: None,
            alive: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            rx: None,
        })
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.errors.clear();
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
        let return_type = self.return_type.clone();
        let (tx, rx) = unbounded();
        self.rx = Some(rx);
        self.alive.store(true, Ordering::Relaxed);
        self.stop.store(false, Ordering::Relaxed);
        let alive = self.alive.clone();
        let stop = self.stop.clone();
        let duration = self.duration.clone();
        self.thr = Some(thread::spawn(move || {
            let start_time = Instant::now();
            entries_thread(
                root_path,
                sorted,
                skip_hidden,
                max_depth,
                max_file_cnt,
                filter,
                return_type,
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

    fn receive_all(&mut self) -> (Vec<ScandirResult>, Vec<(String, String)>) {
        let mut entries: Vec<ScandirResult> = Vec::new();
        let mut errors: Vec<(String, String)> = Vec::new();
        if let Some(ref rx) = self.rx {
            loop {
                match rx.try_recv() {
                    Ok(entry) => match entry.entry {
                        Stats::ScandirResult(ref r) => match r {
                            ScandirResult::DirEntry(_) => entries.push(r.clone()),
                            ScandirResult::DirEntryExt(_) => entries.push(r.clone()),
                            ScandirResult::Error((path, e)) => {
                                errors.push((path.to_owned(), e.to_owned()));
                            }
                        },
                        Stats::Error(e) => {
                            errors.push((entry.path, e));
                        }
                        Stats::Duration(d) => *self.duration.lock().unwrap() = d,
                    },
                    Err(_) => break,
                }
            }
        }
        (entries, errors)
    }

    pub fn collect(&mut self) -> (Vec<ScandirResult>, Vec<(String, String)>) {
        if !self.finished() {
            if !self.busy() {
                self.start();
            }
            self.join();
        }
        self.results(true)
    }

    pub fn results(&mut self, return_all: bool) -> (Vec<ScandirResult>, Vec<(String, String)>) {
        let (entries, errors) = self.receive_all();
        self.entries.extend_from_slice(&entries);
        self.errors.extend(errors.clone());
        if return_all {
            return (self.entries.clone(), self.errors.clone());
        }
        (entries, errors)
    }

    pub fn entries(&mut self, return_all: bool) -> Vec<ScandirResult> {
        self.results(return_all).0
    }

    pub fn errors(&mut self, return_all: bool) -> Vec<(String, String)> {
        self.results(return_all).1
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&mut self) -> bool {
        *self.duration.lock().unwrap() > 0.0
    }

    pub fn has_entries(&mut self) -> bool {
        !self.entries.is_empty()
    }

    pub fn has_errors(&mut self) -> bool {
        !self.errors.is_empty()
    }

    pub fn busy(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }
}
