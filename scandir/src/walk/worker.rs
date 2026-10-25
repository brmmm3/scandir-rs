use std::fmt::Debug;
use std::io::Error;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "bincode")]
use bincode_next::error::EncodeError;
use flume::{Receiver, RecvTimeoutError, Sender};
use jwalk_meta::WalkDirGeneric;
use parking_lot::Mutex;
#[cfg(feature = "speedy")]
use speedy::Writable;

use crate::count::Statistics;
use crate::{
    DirEntryType, ErrorsType, Filter, Options, ReturnType, Toc, check_and_expand_path,
    create_filter, filter_children, get_root_path_len, start,
};

/// Accumulated (entries, filter_errors) shared across rayon workers.
/// Wrapped in Arc<Mutex<..>> to halve contention vs separate locks.
type WalkResults = Arc<Mutex<(Vec<(String, Toc)>, Vec<String>)>>;

#[inline]
fn update_toc(dir_entry: &DirEntryType, toc: &mut Toc) {
    let file_type = dir_entry.file_type;
    let key = dir_entry.file_name.to_string_lossy().into_owned();
    // is_file first — most common case, reduces branch misprediction
    if file_type.is_file() {
        toc.files.push(key);
    } else if file_type.is_dir() {
        toc.dirs.push(key);
    } else if file_type.is_symlink() {
        toc.symlinks.push(key);
    } else {
        toc.other.push(key);
    }
}

/// Streaming mode worker: sends (directory, Toc) tuples through channel.
fn worker_thread(
    dir_entry: DirEntryType,
    options: Options,
    filter: Option<Filter>,
    tx: Sender<(String, Toc)>,
    stop: Arc<AtomicBool>,
) -> Result<(), Error> {
    let root_path_len = get_root_path_len(&options.root_path)?;
    // If root path points to a file then return just this one entry
    if !dir_entry.file_type.is_dir() {
        let mut toc = Toc::new();

        update_toc(&dir_entry, &mut toc);
        if tx.send(("".to_owned(), toc)).is_err() {
            stop.store(true, Ordering::Relaxed);
        }
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
        .process_read_dir(move |_, root_dir, _, children| {
            if stop_cb.load(Ordering::Relaxed) {
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
            let mut toc = Toc::new();
            toc.errors
                .extend(filter_children(children, &filter, root_path_len));
            children
                .iter_mut()
                .for_each(|dir_entry_result| match dir_entry_result {
                    Ok(dir_entry) => update_toc(dir_entry, &mut toc),
                    Err(e) => {
                        if toc.errors.len() < 1000 {
                            toc.errors.push(e.to_string());
                        }
                    }
                });
            let msg = if root_dir.len() > root_path_len {
                (root_dir[root_path_len..].to_owned(), toc)
            } else {
                (String::new(), toc)
            };
            if tx.send(msg).is_err() {
                stop_cb.store(true, Ordering::Relaxed);
            }
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
                // H3: outer iterator yielded I/O error (root path / boundary).
                // process_read_dir callback handles per-entry errors; forward
                // this rare outer error as an errors-only Toc.
                let mut err_toc = Toc::new();
                err_toc.errors.push(e.to_string());
                if tx_outer.send((String::new(), err_toc)).is_err() {
                    break;
                }
            }
        }
    }
    Ok(())
}

/// Direct mode worker: accumulates Toc results in-process, returns via JoinHandle.
/// Eliminates channel overhead — results accumulate in a Vec guarded by a Mutex,
/// which grows unbounded with directory size (caller-side bounded only by max_file_cnt).
/// Uses Arc<Mutex<Vec>> to share results between process_read_dir callback and this function.
fn worker_thread_direct(
    dir_entry: DirEntryType,
    options: Options,
    filter: Option<Filter>,
    stop: Arc<AtomicBool>,
) -> Result<Vec<(String, Toc)>, Error> {
    let root_path_len = get_root_path_len(&options.root_path)?;
    // If root path points to a file then return just this one entry
    if !dir_entry.file_type.is_dir() {
        let mut toc = Toc::new();
        update_toc(&dir_entry, &mut toc);
        return Ok(vec![("".to_owned(), toc)]);
    }

    let max_file_cnt = options.max_file_cnt;
    let mut file_cnt = 0;

    // Entries + filter errors share one lock to halve contention on the
    // rayon-parallel process_read_dir callback. parking_lot::Mutex has no
    // poisoning, so locks return a guard directly.
    let results: WalkResults = Arc::new(parking_lot::Mutex::new((Vec::new(), Vec::new())));
    let results_clone = results.clone();
    let stop_cb = stop.clone();

    for result in WalkDirGeneric::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .follow_links(options.follow_links)
        .sort(options.sorted)
        .max_depth(options.max_depth)
        .process_read_dir(move |_, root_dir, _, children| {
            if stop_cb.load(Ordering::Relaxed) {
                return;
            }
            let root_dir_str = root_dir.to_str();
            if root_dir_str.is_none() {
                return;
            }
            let root_dir_str = root_dir_str.unwrap();
            if root_dir_str.len() + 1 < root_path_len {
                return;
            }
            let errs = filter_children(children, &filter, root_path_len);
            let mut toc = Toc::new();
            children
                .iter_mut()
                .for_each(|dir_entry_result| match dir_entry_result {
                    Ok(dir_entry) => update_toc(dir_entry, &mut toc),
                    Err(e) => {
                        if toc.errors.len() < 1000 {
                            toc.errors.push(e.to_string());
                        }
                    }
                });
            let key = if root_dir_str.len() > root_path_len {
                root_dir_str[root_path_len..].to_owned()
            } else {
                String::new()
            };
            // Single lock — push entries and filter errors together
            let mut guard = results_clone.lock();
            if !errs.is_empty() && guard.1.len() < 1000 {
                guard.1.extend(errs);
            }
            guard.0.push((key, toc));
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
                // H3: forward outer iterator error into filter_errors (capped at 1000).
                let mut guard = results.lock();
                if guard.1.len() < 1000 {
                    guard.1.push(e.to_string());
                }
            }
        }
    }

    // Merge filter errors into the root Toc, then take entries (single lock).
    // Arc refcount may be >1: jwalk-meta's iterator may retain the
    // process_read_dir closure. Lock+take avoids try_unwrap assumption.
    let mut guard = results.lock();
    if !guard.1.is_empty() {
        let errs: Vec<String> = std::mem::take(&mut guard.1);
        if let Some(first) = guard.0.first_mut() {
            first.1.errors.extend(errs);
        } else {
            let mut toc = Toc::new();
            toc.errors.extend(errs);
            guard.0.push((String::new(), toc));
        }
    }
    Ok(std::mem::take(&mut guard.0))
}

#[derive(Debug)]
pub struct Walk {
    // Options
    options: Options,
    store: bool,
    // Results
    entries: Vec<(String, Toc)>,
    duration: Arc<Mutex<f64>>,
    finished: Arc<AtomicBool>,
    has_errors: bool,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<(String, Toc)>>,
}

impl Walk {
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(
        root_path: P,
        store: Option<bool>,
    ) -> Result<Self, Error> {
        Ok(Walk {
            options: Options {
                root_path: check_and_expand_path(root_path)?,
                sorted: false,
                skip_hidden: true,
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
            entries: Vec::new(),
            duration: Arc::new(Mutex::new(0.0)),
            finished: Arc::new(AtomicBool::new(false)),
            has_errors: false,
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

    /// Set extended return type
    pub fn return_type(mut self, return_type: ReturnType) -> Self {
        self.options.return_type = return_type;
        self
    }

    /// Set extended return type
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
        if self.busy() {
            return;
        }
        self.entries.clear();
        self.has_errors = false;
        *self.duration.lock() = 0.0;
    }

    pub fn start(&mut self) -> Result<(), Error> {
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

    fn receive_all(&mut self) -> Vec<(String, Toc)> {
        let mut entries = Vec::new();
        if let Some(ref rx) = self.rx {
            entries.reserve(rx.len());
            while let Ok(entry) = rx.try_recv() {
                if !entry.1.errors.is_empty() {
                    self.has_errors = true;
                }
                entries.push(entry);
            }
        }
        entries
    }

    /// Receive results with a timeout. Waits up to `timeout` for the first result,
    /// then drains all available results from the channel.
    /// Returns an empty Vec if timeout expires before any data arrives.
    fn receive_all_timeout(&mut self, timeout: Duration) -> Vec<(String, Toc)> {
        // Fast path: drain any immediately-available results.
        let mut entries = self.receive_all();
        if !entries.is_empty() {
            return entries;
        }
        if let Some(ref rx) = self.rx {
            // Channel empty — wait for the first result, then drain again.
            match rx.recv_timeout(timeout) {
                Ok(entry) => {
                    if !entry.1.errors.is_empty() {
                        self.has_errors = true;
                    }
                    entries.push(entry);
                    // Drain remaining non-blocking.
                    let extra = self.receive_all();
                    entries.extend(extra);
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    // H2: worker thread panicked — channel disconnected.
                    // Mark finished so outer busy()/finished() loops exit instead of spinning.
                    self.finished.store(true, Ordering::Relaxed);
                }
            }
        }
        entries
    }

    /// Collect all results using direct mode (no channel overhead).
    /// Worker thread accumulates results in-process and returns via JoinHandle.
    /// This eliminates channel memory accumulation that caused unbounded growth.
    pub fn collect(&mut self) -> Result<Toc, Error> {
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

        let raw_entries = handle
            .join()
            .map_err(|_| Error::other("Worker thread panicked"))??;

        // bs1: write back to self so collect mode API (has_errors/errors_cnt/errors/statistics) works
        self.entries = raw_entries;
        self.has_errors = self.entries.iter().any(|(_, t)| !t.errors.is_empty());

        // Merge all per-directory Tocs into a single Toc
        let mut toc = Toc::new();
        for (root_dir, dir_toc) in &self.entries {
            toc.extend(root_dir, dir_toc);
        }
        Ok(toc)
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
        } else {
            self.entries.len()
        }
    }

    pub fn results(&mut self, only_new: bool) -> Vec<(String, Toc)> {
        let entries = self.receive_all();
        if self.store {
            self.entries.extend_from_slice(&entries);
        }
        if !only_new && self.store {
            return std::mem::take(&mut self.entries);
        }
        entries
    }

    /// Retrieve results with a timeout for event-driven consumption.
    /// Waits up to `timeout` for results to arrive when channel is empty.
    /// Returns new results only (always only_new=true behavior).
    pub fn results_timeout(&mut self, timeout: Duration) -> Vec<(String, Toc)> {
        let entries = self.receive_all_timeout(timeout);
        if self.store {
            self.entries.extend_from_slice(&entries);
        }
        entries
    }

    /// Collect results with an optional timeout for time-bounded operation.
    /// Returns `Ok(Some(Toc))` when collection completes within timeout.
    /// Returns `Ok(None)` if timeout expires before completion.
    /// Returns `Ok(Some(Toc))` immediately if already finished.
    /// The worker thread continues running after timeout — call `stop()` to abort.
    pub fn collect_timeout(&mut self, timeout: Duration) -> Result<Option<Toc>, Error> {
        if !self.finished() {
            if !self.busy() {
                self.start()?;
            }
            let deadline = Instant::now() + timeout;
            while !self.finished() && self.busy() {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    // Timeout expired, return partial results
                    let mut toc = Toc::new();
                    for (root_dir, dir_toc) in self.results(false) {
                        toc.extend(&root_dir, &dir_toc);
                    }
                    return Ok(None);
                }
                let entries = self.receive_all_timeout(remaining.min(Duration::from_millis(100)));
                if self.store {
                    self.entries.extend_from_slice(&entries);
                }
            }
        }
        let mut toc = Toc::new();
        for (root_dir, dir_toc) in self.results(false) {
            toc.extend(&root_dir, &dir_toc);
        }
        Ok(Some(toc))
    }

    pub fn has_errors(&mut self) -> bool {
        self.has_errors
    }

    pub fn errors_cnt(&mut self) -> usize {
        self.entries.iter().map(|e| e.1.errors.len()).sum()
    }

    pub fn errors(&mut self, only_new: bool) -> ErrorsType {
        self.results(only_new)
            .iter()
            .flat_map(|e| {
                e.1.errors
                    .iter()
                    .map(|err| (e.0.clone(), err.to_string()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.entries.write_to_vec()
    }

    #[cfg(feature = "bincode")]
    pub fn to_bincode(&self) -> Result<Vec<u8>, EncodeError> {
        bincode_next::serde::encode_to_vec(&self.entries, bincode_next::config::legacy())
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.entries)
    }

    pub fn statistics(&self) -> Statistics {
        let mut statistics = Statistics::new();
        for (_dir, toc) in self.entries.iter() {
            statistics.dirs += toc.dirs.len() as i32;
            statistics.files += toc.files.len() as i32;
            statistics.slinks += toc.symlinks.len() as i32;
            statistics.devices += toc.other.len() as i32;
            statistics.errors.extend_from_slice(&toc.errors);
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
