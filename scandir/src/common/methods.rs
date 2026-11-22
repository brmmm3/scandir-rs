use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use std::{fs, thread};

#[cfg(unix)]
use expanduser::expanduser;

use flume::{Receiver, Sender, bounded};
use glob_sl::{MatchOptions, Pattern};
use parking_lot::Mutex;

use crate::{DirEntryType, Filter, Options};

pub(crate) fn claim_file_slot(count: &AtomicUsize, limit: usize) -> bool {
    if limit == usize::MAX {
        return true;
    }
    let mut current = count.load(Ordering::Relaxed);
    while current < limit {
        match count.compare_exchange_weak(
            current,
            current + 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return true,
            Err(actual) => current = actual,
        }
    }
    false
}

pub fn check_and_expand_path<P>(path_str: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let path = path_str.as_ref();
    #[cfg(unix)]
    let path = if let Some(first) = path
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        && first.starts_with('~')
    {
        let expanded =
            expanduser(first).map_err(|e| Error::other(format!("Invalid path {path:?}: {e:?}")))?;
        expanded.join(path.strip_prefix(first).map_err(Error::other)?)
    } else {
        path.to_path_buf()
    };
    fs::canonicalize(path)
}

pub fn get_root_path_len(root_path: &Path) -> Result<usize, Error> {
    let root_path = match root_path.to_str() {
        Some(v) => v,
        None => return Err(Error::other(format!("Invalid path: {root_path:?}"))),
    };
    let mut root_path_len = root_path.len();
    #[cfg(unix)]
    if !root_path.ends_with('/') {
        root_path_len += 1;
    }
    #[cfg(windows)]
    if !root_path.ends_with('\\') {
        root_path_len += 1;
    }
    Ok(root_path_len)
}

pub fn create_filter(options: &Options) -> Result<Option<Filter>, Error> {
    let mut filter = Filter {
        dir_include: Vec::new(),
        dir_exclude: Vec::new(),
        file_include: Vec::new(),
        file_exclude: Vec::new(),
        options: match options.case_sensitive {
            true => None,
            false => Some(MatchOptions {
                case_sensitive: false,
                ..MatchOptions::new()
            }),
        },
    };
    if let Some(ref f) = options.dir_include {
        let f = &mut f
            .iter()
            .map(|s| Pattern::new(s))
            .collect::<Result<Vec<_>, glob_sl::PatternError>>();
        let f = match f {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("dir_include: {}", e),
                ));
            }
        };
        filter.dir_include.append(f);
    }
    if let Some(ref f) = options.dir_exclude {
        let f = &mut f
            .iter()
            .map(|s| Pattern::new(s))
            .collect::<Result<Vec<_>, glob_sl::PatternError>>();
        let f = match f {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("dir_exclude: {}", e),
                ));
            }
        };
        filter.dir_exclude.append(f);
    }
    if let Some(ref f) = options.file_include {
        let f = &mut f
            .iter()
            .map(|s| Pattern::new(s))
            .collect::<Result<Vec<_>, glob_sl::PatternError>>();
        let f = match f {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("file_include: {}", e),
                ));
            }
        };
        filter.file_include.append(f);
    }
    if let Some(ref f) = options.file_exclude {
        let f = &mut f
            .iter()
            .map(|s| Pattern::new(s))
            .collect::<Result<Vec<_>, glob_sl::PatternError>>();
        let f = match f {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("file_exclude: {}", e),
                ));
            }
        };
        filter.file_exclude.append(f);
    }
    if filter.dir_include.is_empty()
        && filter.dir_exclude.is_empty()
        && filter.file_include.is_empty()
        && filter.file_exclude.is_empty()
    {
        return Ok(None);
    }
    Ok(Some(filter))
}

#[inline]
pub fn filter_direntry(
    key: &str,
    filter: &Vec<Pattern>,
    options: Option<MatchOptions>,
    empty: bool,
) -> bool {
    if filter.is_empty() || key.is_empty() {
        return empty;
    }
    match options {
        Some(options) => {
            for f in filter {
                if f.as_str().ends_with("**") && !key.ends_with('/') {
                    // Workaround: glob currently has problems with "foo/**"
                    let mut key = String::from(key);
                    key.push('/');
                    if f.matches_with(&key, options) {
                        return true;
                    }
                }
                if f.matches_with(key, options) {
                    return true;
                }
            }
        }
        None => {
            for f in filter {
                if f.as_str().ends_with("**") && !key.ends_with('/') {
                    // Workaround: glob currently has problems with "foo/**"
                    let mut key = String::from(key);
                    key.push('/');
                    if f.matches(&key) {
                        return true;
                    }
                }
                if f.matches(key) {
                    return true;
                }
            }
        }
    }
    false
}

#[inline]
pub fn filter_dir(
    root_path: &Path,
    dir_entry: &DirEntryType,
    filter_ref: &Filter,
) -> Result<bool, String> {
    if filter_ref.dir_include.is_empty() && filter_ref.dir_exclude.is_empty() {
        return Ok(true);
    }
    let parent = dir_entry
        .parent_path
        .strip_prefix(root_path)
        .map_err(|_| format!("Directory outside scan root: {:?}", dir_entry.parent_path))?;
    let path = parent.join(&dir_entry.file_name);
    let key = path
        .to_str()
        .ok_or_else(|| format!("Cannot apply Unicode directory filter to {path:?}"))?;
    #[cfg(windows)]
    let key = &key.replace('\\', "/");
    Ok(
        !(filter_direntry(key, &filter_ref.dir_exclude, filter_ref.options, false)
            || !filter_direntry(key, &filter_ref.dir_include, filter_ref.options, true)),
    )
}

#[inline]
pub fn filter_children(
    children: &mut Vec<Result<DirEntryType, jwalk_meta::Error>>,
    filter: &Option<Filter>,
    root_path: &Path,
) -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(filter_ref) = &filter {
        children.retain(|dir_entry_result| {
            dir_entry_result
                .as_ref()
                .map(|dir_entry| {
                    if dir_entry.file_type.is_dir() {
                        return match filter_dir(root_path, dir_entry, filter_ref) {
                            Ok(include) => include,
                            Err(error) => {
                                errors.push(error);
                                false
                            }
                        };
                    } else {
                        if filter_ref.file_include.is_empty() && filter_ref.file_exclude.is_empty()
                        {
                            return true;
                        }
                        let options = filter_ref.options;
                        let key = match dir_entry.file_name.to_str() {
                            Some(s) => s,
                            None => {
                                errors.push(format!(
                                    "Cannot apply Unicode file filter to {:?}",
                                    dir_entry.file_name
                                ));
                                return false;
                            }
                        };
                        if filter_direntry(key, &filter_ref.file_exclude, options, false)
                            || !filter_direntry(key, &filter_ref.file_include, options, true)
                        {
                            return false;
                        }
                    }
                    true
                })
                .unwrap_or_else(|e| {
                    errors.push(e.to_string());
                    false
                })
        });
    }
    errors
}

#[allow(clippy::type_complexity)]
pub fn start<T: Send + 'static + std::fmt::Debug>(
    options: Options,
    duration: Arc<Mutex<f64>>,
    finished: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    worker_thread: fn(
        DirEntryType,
        Options,
        Option<Filter>,
        Sender<T>,
        Arc<AtomicBool>,
    ) -> Result<(), Error>,
) -> Result<(Option<thread::JoinHandle<()>>, Option<Receiver<T>>), Error> {
    let filter = create_filter(&options)?;
    let (tx, rx) = bounded(16384);
    stop.store(false, Ordering::Relaxed);
    // Create root DirEntry here, so that errors are immediately returned
    let dir_entry: DirEntryType = jwalk_meta::DirEntry::from_path(
        0,
        &options.root_path,
        true,
        true,
        options.follow_links,
        Some(Arc::new(Vec::new())),
    )?;
    Ok((
        Some(thread::spawn(move || {
            let start_time = Instant::now();
            let _ = worker_thread(dir_entry, options, filter, tx, stop);
            *duration.lock() = start_time.elapsed().as_secs_f64();
            finished.store(true, Ordering::Relaxed);
        })),
        Some(rx),
    ))
}
