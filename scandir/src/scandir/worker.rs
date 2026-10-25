use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

#[cfg(feature = "bincode")]
use bincode_next::error::EncodeError;
use flume::{Receiver, RecvTimeoutError, Sender};

use jwalk_meta::WalkDirGeneric;
use parking_lot::Mutex;

use crate::count::Statistics;
use crate::scandir::{ScandirResult, ScandirResults};
use crate::{
    DirEntry, DirEntryExt, DirEntryType, ErrorsType, Filter, Options, ReturnType,
    check_and_expand_path, create_filter, filter_children, get_root_path_len, start,
};

#[inline]
fn create_entry(
    root_path_len: usize,
    return_type: &ReturnType,
    dir_entry: &DirEntryType,
) -> ScandirResult {
    let file_type = dir_entry.file_type;
    let mut st_ctime: Option<SystemTime> = None;
    let mut st_mtime: Option<SystemTime> = None;
    let mut st_atime: Option<SystemTime> = None;
    let mut st_btime: Option<SystemTime> = None;
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
        st_ctime = metadata.changed;
        st_btime = metadata.created;
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
    let path_str = dir_entry.parent_path.to_string_lossy();
    let file_name = dir_entry.file_name.to_string_lossy();
    let path_str: &str = &path_str;
    let file_name: &str = &file_name;
    let path = if path_str.len() > root_path_len {
        let relative = &path_str[root_path_len..];
        let mut p = String::with_capacity(relative.len() + 1 + file_name.len());
        p.push_str(relative);
        p.push('/');
        p.push_str(file_name);
        p
    } else {
        file_name.to_owned()
    };
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
            // Base branch above moves `path`, Ext branch must re-build it.
            // Returned by the closure as the last statement, no shared use beyond this fn.
            path: if path_str.len() > root_path_len {
                let relative = &path_str[root_path_len..];
                let mut p = String::with_capacity(relative.len() + 1 + file_name.len());
                p.push_str(relative);
                p.push('/');
                p.push_str(file_name);
                p
            } else {
                file_name.to_owned()
            },
            is_symlink: file_type.is_symlink(),
            is_dir: file_type.is_dir(),
            is_file,
            st_ctime,
            st_mtime,
            st_atime,
            st_btime,
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

/// Streaming mode worker: sends entries through channel for incremental consumption.
fn worker_thread(
    dir_entry: DirEntryType,
    options: Options,
    filter: Option<Filter>,
    tx: Sender<ScandirResult>,
    stop: Arc<AtomicBool>,
) -> Result<(), Error> {
    let root_path_len = get_root_path_len(&options.root_path)?;
    let return_type = options.return_type;
    // If root path points to a file then return just this one entry
    if !dir_entry.file_type.is_dir() {
        let _ = tx.send(create_entry(root_path_len, &return_type, &dir_entry));
        return Ok(());
    }

    let max_file_cnt = options.max_file_cnt;
    let mut file_cnt = 0;
    let stop_cb = stop.clone();
    let tx_outer = tx.clone(); // H3: outer for-loop needs Sender; process_read_dir closure still moves original `tx`

    for result in WalkDirGeneric::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .follow_links(options.follow_links)
        .sort(options.sorted)
        .max_depth(options.max_depth)
        .read_metadata(true)
        .read_metadata_ext(options.return_type == ReturnType::Ext)
        .read_hardlink_info(options.return_type == ReturnType::Ext)
        .process_read_dir(move |_, root_dir, _, children| {
            if stop_cb.load(Ordering::Relaxed) {
                return;
            }
            if let Some(root_dir) = root_dir.to_str() {
                if root_dir.len() + 1 < root_path_len {
                    return;
                }
            } else {
                return;
            }
            for e in filter_children(children, &filter, root_path_len) {
                if tx.send(ScandirResult::Error((String::new(), e))).is_err() {
                    stop_cb.store(true, Ordering::Relaxed);
                    return;
                }
            }
            #[allow(clippy::needless_return)]
            children
                .iter_mut()
                .for_each(|dir_entry_result| match dir_entry_result {
                    Ok(dir_entry) => {
                        if tx
                            .send(create_entry(root_path_len, &return_type, dir_entry))
                            .is_err()
                        {
                            stop_cb.store(true, Ordering::Relaxed);
                            return;
                        }
                    }
                    Err(e) => {
                        if tx
                            .send(ScandirResult::Error((String::new(), e.to_string())))
                            .is_err()
                        {
                            stop_cb.store(true, Ordering::Relaxed);
                            return;
                        }
                    }
                });
        })
    {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        match result {
            Ok(dir_entry) => {
                if !dir_entry.file_type.is_dir() {
                    file_cnt += 1;
                    if max_file_cnt > 0 && file_cnt > max_file_cnt {
                        break;
                    }
                }
            }
            Err(e) => {
                // H3: outer iterator yielded I/O error — forward as ScandirResult::Error
                // (same pattern as process_read_dir callback at line 195).
                if tx_outer
                    .send(ScandirResult::Error((String::new(), e.to_string())))
                    .is_err()
                {
                    break;
                }
            }
        }
    }
    Ok(())
}

/// Direct mode worker: accumulates results in-process, returns via JoinHandle.
/// Eliminates channel overhead — zero additional memory for channel buffering.
fn worker_thread_direct(
    dir_entry: DirEntryType,
    options: Options,
    filter: Option<Filter>,
    stop: Arc<AtomicBool>,
) -> Result<ScandirResults, Error> {
    let root_path_len = get_root_path_len(&options.root_path)?;
    let return_type = options.return_type;

    // If root path points to a file then return just this one entry
    if !dir_entry.file_type.is_dir() {
        let mut results = ScandirResults::new();
        results.push_entry(create_entry(root_path_len, &return_type, &dir_entry));
        return Ok(results);
    }

    let max_file_cnt = options.max_file_cnt;
    let mut file_cnt = 0;
    let mut entries = ScandirResults::new();

    // Collect filter errors via Arc<parking_lot::Mutex> (no poisoning)
    let filter_errors: Arc<parking_lot::Mutex<Vec<String>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));
    let filter_errors_clone = filter_errors.clone();

    for result in WalkDirGeneric::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .follow_links(options.follow_links)
        .sort(options.sorted)
        .max_depth(options.max_depth)
        .read_metadata(true)
        .read_metadata_ext(return_type == ReturnType::Ext)
        .read_hardlink_info(return_type == ReturnType::Ext)
        .process_read_dir(move |_, root_dir, _, children| {
            if let Some(root_dir) = root_dir.to_str() {
                if root_dir.len() + 1 < root_path_len {
                    return;
                }
            } else {
                return;
            }
            let errs = filter_children(children, &filter, root_path_len);
            if !errs.is_empty() {
                filter_errors_clone.lock().extend(errs);
            }
            // Only filter children here — do NOT send through channel
            // The outer for loop will yield each DirEntry
        })
    {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        match result {
            Ok(dir_entry) => {
                entries.push_entry(create_entry(root_path_len, &return_type, &dir_entry));
                if !dir_entry.file_type.is_dir() {
                    file_cnt += 1;
                    if max_file_cnt > 0 && file_cnt > max_file_cnt {
                        break;
                    }
                }
            }
            Err(e) => {
                if entries.errors.len() < 1000 {
                    entries.errors.push((String::new(), e.to_string()));
                }
            }
        }
    }

    // Merge filter errors collected in callback
    let mut guard = filter_errors.lock();
    let errs: Vec<String> = guard.drain(..).collect();
    if entries.errors.len() < 1000 {
        entries.errors.extend(
            errs.into_iter()
                .take(1000 - entries.errors.len())
                .map(|e| (String::new(), e)),
        );
    }

    Ok(entries)
}

/// Class for iterating a file tree and returning `Entry` objects
#[derive(Debug)]
pub struct Scandir {
    // Options
    options: Options,
    store: bool,
    // Results
    entries: Arc<ScandirResults>,
    duration: Arc<Mutex<f64>>,
    finished: Arc<AtomicBool>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<ScandirResult>>,
}

impl Scandir {
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(
        root_path: P,
        store: Option<bool>,
    ) -> Result<Self, Error> {
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
            store: store.unwrap_or(false),
            entries: Arc::new(ScandirResults::new()),
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
        Arc::make_mut(&mut self.entries).clear();
        *self.duration.lock() = 0.0;
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.options.return_type > ReturnType::Ext {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Parameter return_type has invalid value",
            ));
        }
        if self.busy() {
            return Err(Error::other("Busy"));
        }
        self.clear();
        (self.thr, self.rx) = start(
            self.options.clone(),
            self.duration.clone(),
            self.finished.clone(),
            self.stop.clone(),
            worker_thread,
        )?;
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

    /// Collect all results using direct mode (no channel overhead).
    /// Worker thread accumulates results in-process and returns via JoinHandle.
    /// This eliminates channel memory accumulation that caused unbounded growth.
    pub fn collect(&mut self) -> Result<Arc<ScandirResults>, Error> {
        if self.options.return_type > ReturnType::Ext {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Parameter return_type has invalid value",
            ));
        }
        if self.busy() {
            return Err(Error::other("Busy"));
        }
        self.clear();

        let filter = create_filter(&self.options)?;
        let options = self.options.clone();
        let stop = self.stop.clone();
        let duration = self.duration.clone();
        let finished = self.finished.clone();

        stop.store(false, Ordering::Relaxed);
        let dir_entry: DirEntryType = jwalk_meta::DirEntry::from_path(
            0,
            &options.root_path,
            true,
            true,
            options.follow_links,
            Some(Arc::new(Vec::new())),
        )?;

        let start_time = Instant::now();
        let handle = thread::spawn(move || {
            let result = worker_thread_direct(dir_entry, options, filter, stop);
            *duration.lock() = start_time.elapsed().as_secs_f64();
            finished.store(true, Ordering::Relaxed);
            result
        });

        let results = handle
            .join()
            .map_err(|_| Error::other("Worker thread panicked"))??;

        // bs1: write back to self so collect mode API (has_errors/errors_cnt/errors) works
        // Approach D: Share ownership via Arc; O(1) clone instead of deep-cloning 440,000 entries.
        let arc = Arc::new(results);
        self.entries = arc.clone();
        Ok(arc)
    }

    /// Collect results with a timeout for time-bounded operation.
    /// Returns `Ok(Some(ScandirResults))` when collection completes within timeout.
    /// Returns `Ok(None)` if timeout expires before completion.
    /// The worker thread continues running after timeout — call `stop()` to abort.
    pub fn collect_timeout(&mut self, timeout: Duration) -> Result<Option<ScandirResults>, Error> {
        if !self.finished() {
            if !self.busy() {
                self.start()?;
            }
            let deadline = Instant::now() + timeout;
            while !self.finished() && self.busy() {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Ok(None);
                }
                let _ = self.receive_all_timeout(remaining.min(Duration::from_millis(100)));
            }
        }
        Ok(Some(self.results(false)))
    }

    pub fn has_results(&mut self, only_new: bool) -> bool {
        if let Some(ref rx) = self.rx
            && !rx.is_empty()
        {
            return true;
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
            results.results.reserve(rx.len());
            while let Ok(entry) = rx.try_recv() {
                if let ScandirResult::Error(e) = entry {
                    results.errors.push(e);
                } else {
                    results.push_entry(entry);
                }
            }
        }
        if self.store {
            Arc::make_mut(&mut self.entries).extend(&results);
        }
        if !only_new && self.store {
            return std::mem::take(Arc::make_mut(&mut self.entries));
        }
        results
    }

    /// Receive results with a timeout. Waits up to `timeout` for the first result,
    /// then drains all available results from the channel.
    fn receive_all_timeout(&mut self, timeout: Duration) -> ScandirResults {
        let mut results = ScandirResults::new();
        if let Some(ref rx) = self.rx {
            results.results.reserve(rx.len());
            // First, try non-blocking drain
            while let Ok(entry) = rx.try_recv() {
                match entry {
                    ScandirResult::Error(e) => results.errors.push(e),
                    other => results.push_entry(other),
                }
            }
            // If nothing available and worker busy, wait for first result
            if results.is_empty() {
                match rx.recv_timeout(timeout) {
                    Ok(entry) => {
                        match entry {
                            ScandirResult::Error(e) => results.errors.push(e),
                            other => results.push_entry(other),
                        }
                        // Drain remaining
                        while let Ok(entry) = rx.try_recv() {
                            match entry {
                                ScandirResult::Error(e) => results.errors.push(e),
                                other => results.push_entry(other),
                            }
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => {
                        // H2: worker thread panicked — channel disconnected.
                        // Mark finished so outer busy()/finished() loops exit instead of spinning.
                        self.finished.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
        if self.store {
            Arc::make_mut(&mut self.entries).extend(&results);
        }
        results
    }

    /// Retrieve results with a timeout for event-driven consumption.
    /// Waits up to `timeout` for results to arrive when channel is empty.
    /// Returns new results only (always only_new=true behavior).
    pub fn results_timeout(&mut self, timeout: Duration) -> ScandirResults {
        self.receive_all_timeout(timeout)
    }

    pub fn has_entries(&mut self, only_new: bool) -> bool {
        if let Some(ref rx) = self.rx
            && !rx.is_empty()
        {
            return true;
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
    pub fn to_bincode(&self) -> Result<Vec<u8>, EncodeError> {
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
        *self.duration.lock()
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
