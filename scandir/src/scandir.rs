use std::collections::HashSet;
use std::fs::Metadata;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, SystemTime};

use flume::{unbounded, Receiver, Sender};

use jwalk_meta::WalkDirGeneric;

use crate::common::{check_and_expand_path, create_filter, filter_children, get_root_path_len};
use crate::def::scandir::ScandirResults;
use crate::def::{DirEntry, DirEntryExt, ErrorsType, Filter, Options, ReturnType, ScandirResult};
use crate::Statistics;

#[derive(Debug, Clone)]
pub enum Stats {
    ScandirResult(ScandirResult),
    Error(String),
    Duration(f64),
}

#[inline]
fn create_entry(
    root_path_len: usize,
    return_type: &ReturnType,
    dir_entry: &jwalk_meta::DirEntry<((), Option<Result<Metadata, Error>>)>,
) -> ScandirResult {
    let file_type = dir_entry.file_type;
    let mut st_ctime: Option<SystemTime> = None;
    let mut st_mtime: Option<SystemTime> = None;
    let mut st_atime: Option<SystemTime> = None;
    let mut st_mode: u32 = 0;
    let mut st_ino: u64 = 0;
    let mut st_dev: u64 = 0;
    let mut st_nlink: u64 = 0;
    let mut st_size: u64 = 0;
    #[cfg(unix)]
    let mut st_blksize: u64 = 4096;
    #[cfg(windows)]
    let st_blksize: u64 = 4096;
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
    if let Some(ref metadata) = dir_entry.metadata {
        st_ctime = metadata.created;
        st_mtime = metadata.modified;
        st_atime = metadata.accessed;
        st_size = metadata.size;
        if let Some(ref metadata) = dir_entry.metadata_ext {
            #[cfg(unix)]
            {
                st_mode = metadata.st_mode;
                st_ino = metadata.st_ino;
                st_dev = metadata.st_dev;
                st_nlink = metadata.st_nlink;
                st_blksize = metadata.st_blksize;
                st_blocks = metadata.st_blocks;
                st_uid = metadata.st_uid;
                st_gid = metadata.st_gid;
                st_rdev = metadata.st_rdev;
            }
            #[cfg(windows)]
            {
                st_mode = metadata.file_attributes;
                st_blocks = st_size >> 12;
                if st_blocks << 12 < st_size {
                    st_blocks += 1;
                }
                // file_index is saved in st_ino
                if let Some(ino) = metadata.file_index {
                    st_ino = ino;
                }
                // volume_serial_number is saved in st_dev
                if let Some(dev) = metadata.volume_serial_number {
                    st_dev = dev as u64;
                }
                // number_of_links is saved in st_nlink
                if let Some(nlink) = metadata.number_of_links {
                    st_nlink = nlink as u64;
                }
            }
        }
    }
    let is_file = file_type.is_file();
    let path_str = dir_entry.parent_path.to_str().unwrap();
    let mut path = if path_str.len() > root_path_len {
        PathBuf::from(&path_str[root_path_len..])
    } else {
        PathBuf::new()
    };
    path.push(&dir_entry.file_name);
    let entry: ScandirResult = match return_type {
        ReturnType::Base => ScandirResult::DirEntry(DirEntry {
            path: path.to_str().unwrap().to_string(),
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
            st_ctime,
            st_mtime,
            st_atime,
            st_size,
        }),
        ReturnType::Ext => ScandirResult::DirEntryExt(DirEntryExt {
            path: path.to_str().unwrap().to_string(),
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
            st_ctime,
            st_mtime,
            st_atime,
            st_mode,
            st_ino,
            st_dev,
            st_nlink,
            st_size,
            st_blksize,
            st_blocks,
            st_uid,
            st_gid,
            st_rdev,
        }),
    };
    entry
}

fn entries_thread(
    options: Options,
    filter: Option<Filter>,
    tx: Sender<ScandirResult>,
    stop: Arc<AtomicBool>,
) {
    let root_path_len = get_root_path_len(&options.root_path);
    let return_type = options.return_type.clone();

    let dir_entry = jwalk_meta::DirEntry::from_path(
        0,
        &options.root_path,
        true,
        true,
        options.follow_links,
        Arc::new(Vec::new()),
    )
    .unwrap();

    if !dir_entry.file_type.is_dir() {
        let _ = tx.send(create_entry(root_path_len, &return_type, &dir_entry));
        return;
    }

    let max_file_cnt = options.max_file_cnt;
    let mut file_cnt = 0;

    for result in WalkDirGeneric::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .follow_links(options.follow_links)
        .sort(options.sorted)
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
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    if tx
                        .send(create_entry(root_path_len, &return_type, dir_entry))
                        .is_err()
                    {
                        return;
                    }
                }
            });
        })
    {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        if let Ok(dir_entry) = result {
            if !dir_entry.file_type.is_dir() {
                file_cnt += 1;
                if max_file_cnt > 0 && file_cnt > max_file_cnt {
                    break;
                }
            }
        }
    }
}

/// Class for iterating a file tree and returning `Entry` objects
#[derive(Debug)]
pub struct Scandir {
    // Options
    options: Options,
    store: bool,
    // Results
    entries: ScandirResults,
    duration: Arc<Mutex<f64>>,
    finished: Arc<AtomicBool>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<ScandirResult>>,
}

impl Scandir {
    pub fn new<P: AsRef<Path>>(root_path: P, store: Option<bool>) -> Result<Self, Error> {
        Ok(Scandir {
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
            store: store.unwrap_or(true),
            entries: ScandirResults::new(),
            duration: Arc::new(Mutex::new(0.0)),
            finished: Arc::new(AtomicBool::new(false)),
            thr: None,
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

    /// Set extended file type parsing
    pub fn return_type(mut self, return_type: ReturnType) -> Self {
        self.options.return_type = return_type;
        self
    }

    /// Set extended file type parsing
    pub fn extended(mut self, extended: bool) -> Self {
        self.options.return_type = match extended {
            false => ReturnType::Base,
            true => ReturnType::Ext,
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
        self.entries.clear();
        *self.duration.lock().unwrap() = 0.0;
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.busy() {
            return Err(Error::other("Busy"));
        }
        if self.options.return_type > ReturnType::Ext {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Parameter return_type has invalid value",
            ));
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
            entries_thread(options, filter, tx, stop);
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

    pub fn collect(&mut self) -> Result<ScandirResults, Error> {
        if !self.finished() {
            if !self.busy() {
                self.start()?;
            }
            self.join();
        }
        Ok(self.results(true))
    }

    pub fn has_results(&mut self, only_new: bool) -> bool {
        if let Some(ref rx) = self.rx {
            if !rx.is_empty() {
                return true;
            }
        }
        if only_new {
            return false;
        }
        !self.entries.is_empty()
    }

    pub fn results_cnt(&mut self, only_new: bool) -> usize {
        if let Some(ref rx) = self.rx {
            if only_new {
                rx.len()
            } else {
                self.entries.len() + rx.len()
            }
        } else if only_new {
            0
        } else {
            self.entries.len()
        }
    }

    pub fn results(&mut self, only_new: bool) -> ScandirResults {
        let mut results = ScandirResults::new();
        if let Some(ref rx) = self.rx {
            while let Ok(entry) = rx.try_recv() {
                if let ScandirResult::Error(e) = entry {
                    results.errors.push(e);
                } else {
                    results.results.push(entry);
                }
            }
        }
        if self.store {
            self.entries.extend(&results);
        }
        if !only_new && self.store {
            return self.entries.clone();
        }
        results
    }

    pub fn has_entries(&mut self, only_new: bool) -> bool {
        if let Some(ref rx) = self.rx {
            if !rx.is_empty() {
                return true;
            }
        }
        if only_new {
            return false;
        }
        !self.entries.is_empty()
    }

    pub fn entries_cnt(&mut self, only_new: bool) -> usize {
        if let Some(ref rx) = self.rx {
            if only_new {
                return rx.len();
            }
            self.entries.len() + rx.len()
        } else {
            self.entries.len()
        }
    }

    pub fn entries(&mut self, only_new: bool) -> Vec<ScandirResult> {
        self.results(only_new).results
    }

    pub fn has_errors(&mut self) -> bool {
        !self.entries.errors.is_empty()
    }

    pub fn errors_cnt(&mut self) -> usize {
        self.entries.errors.len()
    }

    pub fn errors(&mut self, only_new: bool) -> ErrorsType {
        self.results(only_new).errors
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.entries.to_speedy()
    }

    #[cfg(feature = "bincode")]
    pub fn to_bincode(&self) -> bincode::Result<Vec<u8>> {
        self.entries.to_bincode()
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        self.entries.to_json()
    }

    pub fn statistics(&self) -> Statistics {
        let mut statistics = Statistics::new();
        let mut file_indexes: HashSet<u64> = HashSet::new();
        for entry in self.entries.results.iter() {
            if entry.is_file() {
                statistics.files += 1;
                statistics.size += entry.size();
                if let Some(ext) = entry.ext() {
                    statistics.usage += ext.st_blocks << 9;
                    if ext.st_nlink > 1 {
                        if file_indexes.contains(&ext.st_ino) {
                            statistics.hlinks += 1;
                            statistics.files -= 1;
                        } else {
                            file_indexes.insert(ext.st_ino);
                        }
                    }
                }
            } else if entry.is_dir() {
                statistics.dirs += 1;
                statistics.size += 4096;
                statistics.usage += 4096;
            } else if entry.is_symlink() {
                statistics.slinks += 1;
                statistics.size += 4096;
                statistics.usage += 4096;
            } else {
                #[cfg(unix)]
                if let Some(ext) = entry.ext() {
                    {
                        if ext.st_rdev > 0 {
                            statistics.devices += 1;
                        } else if (ext.st_mode & 4096) != 0 {
                            statistics.pipes += 1;
                        }
                    }
                }
                statistics.size += 4096;
                statistics.usage += 4096;
            }
        }
        statistics
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&mut self) -> bool {
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
