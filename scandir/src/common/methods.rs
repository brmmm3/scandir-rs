use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use std::{fs, thread};

#[cfg(unix)]
use expanduser::expanduser;

use flume::{Receiver, Sender, unbounded};
use glob_sl::{MatchOptions, Pattern};
use parking_lot::Mutex;

use crate::{DirEntryType, Filter, Options};

pub fn check_and_expand_path<P>(path_str: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let path_str = match path_str.as_ref().to_str() {
        Some(v) => v,
        None => return Err(Error::other(format!("Invalid path: {path_str:?}"))),
    };
    #[cfg(unix)]
    let path = match expanduser(path_str) {
        Ok(v) => v,
        Err(e) => return Err(Error::other(format!("Invalid path {path_str:?}: {e:?}"))),
    };
    #[cfg(unix)]
    let path_result = fs::canonicalize(path);
    #[cfg(not(unix))]
    let path_result = fs::canonicalize(&path_str);
    let path = match path_result {
        Ok(p) => {
            if !p.exists() {
                return Err(Error::new(ErrorKind::NotFound, path_str.to_string()));
            }
            p
        }
        Err(e) => {
            return Err(Error::other(e.to_string()));
        }
    };
    Ok(path)
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
pub fn filter_dir(root_path_len: usize, dir_entry: &DirEntryType, filter_ref: &Filter) -> bool {
    let mut key = dir_entry.parent_path.to_path_buf();
    key.push(&dir_entry.file_name);
    let key = match key.to_str() {
        Some(v) => v.get(root_path_len..).unwrap_or("").to_string(),
        None => return false,
    };
    !(filter_direntry(&key, &filter_ref.dir_exclude, filter_ref.options, false)
        || !filter_direntry(&key, &filter_ref.dir_include, filter_ref.options, true))
}

#[inline]
pub fn filter_children(
    children: &mut Vec<Result<DirEntryType, jwalk_meta::Error>>,
    filter: &Option<Filter>,
    root_path_len: usize,
) {
    if let Some(filter_ref) = &filter {
        children.retain(|dir_entry_result| {
            dir_entry_result
                .as_ref()
                .map(|dir_entry| {
                    if dir_entry.file_type.is_dir() {
                        return filter_dir(root_path_len, dir_entry, filter_ref);
                    } else {
                        let options = filter_ref.options;
                        let key = match dir_entry.file_name.to_str() {
                            Some(s) => s,
                            None => {
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
                .unwrap_or(false)
        });
    }
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
    let (tx, rx) = unbounded();
    stop.store(false, Ordering::Relaxed);
    // Create root DirEntry here, so that errors are immediately returned
    let dir_entry: DirEntryType = jwalk_meta::DirEntry::from_path(
        0,
        &options.root_path,
        true,
        true,
        options.follow_links,
        Arc::new(Vec::new()),
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
