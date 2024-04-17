use std::fmt::Debug;
use std::fs::{ self, Metadata };
use std::io::{ Error, ErrorKind };
use std::path::Path;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::sync::{ Arc, Mutex };
use std::thread;
use std::time::Instant;

use flume::{ unbounded, Receiver, Sender };
use jwalk_meta::WalkDirGeneric;
use speedy::Writable;

use crate::common::{ check_and_expand_path, create_filter, filter_children, get_root_path_len };
use crate::def::*;

#[inline]
fn update_toc(
    dir_entry: &jwalk_meta::DirEntry<((), Option<Result<fs::Metadata, Error>>)>,
    toc: &mut Toc
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
    stop: Arc<AtomicBool>
) {
    let root_path_len = get_root_path_len(&options.root_path);

    let dir_entry: jwalk_meta::DirEntry<
        ((), Option<Result<Metadata, Error>>)
    > = jwalk_meta::DirEntry
        ::from_path(0, &options.root_path, true, true, false, Arc::new(Vec::new()))
        .unwrap();

    if !dir_entry.file_type.is_dir() {
        let mut toc = Toc::new();

        update_toc(&dir_entry, &mut toc);
        let _ = tx.send(("".to_owned(), toc));
        return;
    }

    let max_file_cnt = options.max_file_cnt;
    let mut file_cnt = 0;
    for result in WalkDirGeneric::new(&options.root_path)
        .skip_hidden(options.skip_hidden)
        .sort(options.sorted)
        .max_depth(options.max_depth)
        .process_read_dir(move |_, root_dir, _, children| {
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
                    update_toc(dir_entry, &mut toc);
                }
            });
            if !toc.is_empty() {
                if root_dir.len() > root_path_len {
                    let _ = tx.send((root_dir[root_path_len..].to_owned(), toc));
                } else {
                    let _ = tx.send(("".to_owned(), toc));
                }
            }
        }) {
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

#[derive(Debug)]
pub struct Walk {
    // Options
    options: Options,
    store: bool,
    // Results
    entries: Vec<(String, Toc)>,
    duration: Arc<Mutex<f64>>,
    has_errors: bool,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<(String, Toc)>>,
}

impl Walk {
    pub fn new<P: AsRef<Path>>(root_path: P, store: Option<bool>) -> Result<Self, Error> {
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
                return_type: ReturnType::Base,
            },
            store: store.unwrap_or(true),
            entries: Vec::new(),
            duration: Arc::new(Mutex::new(0.0)),
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
        self.entries.clear();
        self.has_errors = false;
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
        self.stop.store(false, Ordering::Relaxed);
        let stop = self.stop.clone();
        let duration = self.duration.clone();
        self.thr = Some(
            thread::spawn(move || {
                let start_time = Instant::now();
                toc_thread(options, filter, tx, stop);
                *duration.lock().unwrap() = start_time.elapsed().as_secs_f64();
            })
        );
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
            while let Ok(entry) = rx.try_recv() {
                if !entry.1.errors.is_empty() {
                    self.has_errors = true;
                }
                entries.push(entry);
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
            if only_new { rx.len() } else { self.entries.len() + rx.len() }
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
            return self.entries.clone();
        }
        entries
    }

    pub fn has_errors(&mut self) -> bool {
        !self.has_errors
    }

    pub fn errors_cnt(&mut self) -> usize {
        self.entries
            .iter()
            .map(|e| e.1.errors.len())
            .sum()
    }

    pub fn errors(&mut self, only_new: bool) -> ErrorsType {
        self.results(only_new)
            .iter()
            .flat_map(|e|
                e.1.errors
                    .iter()
                    .map(|err| (e.0.clone(), err.to_string()))
                    .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>()
    }

    #[cfg(feature = "speedy")]
    pub fn to_speedy(&self) -> Result<Vec<u8>, speedy::Error> {
        self.entries.write_to_vec()
    }

    #[cfg(feature = "bincode")]
    pub fn to_bincode(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self.entries)
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
            statistics.errors.extend(toc.errors.clone());
        }
        statistics
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&mut self) -> bool {
        *self.duration.lock().unwrap() > 0.0
    }

    pub fn busy(&self) -> bool {
        if let Some(ref thr) = self.thr { !thr.is_finished() } else { false }
    }

    // For debugging

    pub fn options(&self) -> Options {
        self.options.clone()
    }
}
