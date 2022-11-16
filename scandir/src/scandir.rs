use std::fs::Metadata;
use std::io::{Error, ErrorKind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, UNIX_EPOCH};

use flume::{unbounded, Receiver, Sender};

use jwalk::WalkDirGeneric;

use crate::common::{check_and_expand_path, create_filter, filter_children, get_root_path_len};
use crate::def::{DirEntry, DirEntryExt, Filter, Options, ReturnType, ScandirResult, ScandirResultsType, ErrorsType};

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
        let duration = metadata
            .created
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        st_ctime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let duration = metadata
            .modified
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        st_mtime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let duration = metadata
            .accessed
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_err| Duration::new(0, 0));
        st_atime = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
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
                if let Some(ino) = metadata.file_index {
                    st_ino = ino;
                }
                if let Some(dev) = metadata.volume_serial_number {
                    st_dev = dev as u64;
                }
                if let Some(nlink) = metadata.number_of_links {
                    st_nlink = nlink as u64;
                }
            }
        }
    }
    let is_file = file_type.is_file();
    let mut key = dir_entry.parent_path.to_path_buf();
    let file_name = match dir_entry.file_name.clone().into_string() {
        Ok(s) => s,
        Err(_) => {
            return (
                is_file,
                Entry {
                    path: key.to_str().unwrap().to_string(), // Absolute file path
                    entry: Stats::ScandirResult(ScandirResult::Error((
                        format!("{:?}", dir_entry.file_name),
                        "Invalid file name!".to_string(),
                    ))),
                },
            );
        }
    };
    key.push(&file_name);
    let key = key.to_str().unwrap().to_string();
    let path = key.get(root_path_len..).unwrap_or(&file_name).to_string();
    let entry: ScandirResult = match return_type {
        ReturnType::Base => ScandirResult::DirEntry(DirEntry {
            path,
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
            st_ctime,
            st_mtime,
            st_atime,
            st_size,
        }),
        ReturnType::Ext => ScandirResult::DirEntryExt(DirEntryExt {
            path,
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
    options: Options,
    filter: Option<Filter>,
    tx: Sender<Entry>,
    stop: Arc<AtomicBool>,
) {
    let root_path_len = get_root_path_len(&options.root_path);
    let max_file_cnt = options.max_file_cnt;
    let return_type = options.return_type.clone();
    let file_cnt = Arc::new(AtomicUsize::new(0));
    let file_cnt_cloned = file_cnt.clone();
    let stop_cloned = stop.clone();
    let tx_cloned = tx;
    for _ in WalkDirGeneric::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .sort(options.sorted)
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
            let mut local_file_cnt: usize = 0;
            children.iter_mut().for_each(|dir_entry_result| {
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
    options: Options,
    // Results
    entries: ScandirResultsType,
    errors: ErrorsType,
    duration: Arc<Mutex<f64>>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<Entry>>,
}

impl Scandir {
    pub fn new(root_path: &str) -> Result<Self, Error> {
        Ok(Scandir {
            options: Options {
                root_path: check_and_expand_path(root_path)?,
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
            entries: Vec::new(),
            errors: Vec::new(),
            duration: Arc::new(Mutex::new(0.0)),
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
        self.errors.clear();
        *self.duration.lock().unwrap() = 0.0;
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.busy() {
            return Err(Error::new(ErrorKind::Other, "Busy"));
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
        self.thr = Some(thread::spawn(move || {
            let start_time = Instant::now();
            entries_thread(options, filter, tx, stop);
            *duration.lock().unwrap() = start_time.elapsed().as_secs_f64();
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

    fn receive_all(&mut self) -> (ScandirResultsType, ErrorsType) {
        let mut entries: ScandirResultsType = Vec::new();
        let mut errors: ErrorsType = Vec::new();
        if let Some(ref rx) = self.rx {
            while let Ok(entry) = rx.try_recv() {
                match entry.entry {
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
                }
            }
        }
        (entries, errors)
    }

    pub fn collect(&mut self) -> Result<(ScandirResultsType, ErrorsType), Error> {
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
        !self.entries.is_empty() && !self.errors.is_empty()
    }

    pub fn results_cnt(&mut self, update: bool) -> usize {
        if update {
            self.results(false);
        }
        self.entries.len() + self.errors.len()
    }

    pub fn results(&mut self, return_all: bool) -> (ScandirResultsType, ErrorsType) {
        let (entries, errors) = self.receive_all();
        self.entries.extend_from_slice(&entries);
        self.errors.extend(errors.clone());
        if return_all {
            return (self.entries.clone(), self.errors.clone());
        }
        (entries, errors)
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

    pub fn entries_cnt(&mut self, update: bool) -> usize {
        if update {
            self.results(false);
        }
        self.entries.len()
    }

    pub fn entries(&mut self, return_all: bool) -> ScandirResultsType {
        self.results(return_all).0
    }

    pub fn has_errors(&mut self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors_cnt(&mut self, update: bool) -> usize {
        if update {
            self.results(false);
        }
        self.errors.len()
    }

    pub fn errors(&mut self, return_all: bool) -> ErrorsType {
        self.results(return_all).1
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&mut self) -> bool {
        *self.duration.lock().unwrap() > 0.0
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
