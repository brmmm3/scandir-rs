use std::fs;
use std::fs::Metadata;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use expanduser::expanduser;

use glob::{MatchOptions, Pattern};

use crate::def::{Filter, Options};

pub fn check_and_expand_path(path_str: &str) -> Result<PathBuf, Error> {
    #[cfg(unix)]
    let path_result = fs::canonicalize(expanduser(path_str).unwrap());
    #[cfg(not(unix))]
    let path_result = fs::canonicalize(path_str);
    let path = match path_result {
        Ok(p) => {
            if !p.exists() {
                return Err(Error::new(ErrorKind::NotFound, String::from(path_str)));
            }
            p
        }
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };
    Ok(path)
}

pub fn get_root_path_len(root_path: &Path) -> usize {
    let root_path = root_path.to_str().unwrap();
    let mut root_path_len = root_path.len();
    #[cfg(unix)]
    if !root_path.ends_with("/") {
        root_path_len += 1;
    }
    #[cfg(windows)]
    if !root_path.ends_with('\\') {
        root_path_len += 1;
    }
    root_path_len
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
            .collect::<Result<Vec<_>, glob::PatternError>>();
        let f = match f {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("dir_include: {}", e),
                ))
            }
        };
        filter.dir_include.append(f);
    }
    if let Some(ref f) = options.dir_exclude {
        let f = &mut f
            .iter()
            .map(|s| Pattern::new(s))
            .collect::<Result<Vec<_>, glob::PatternError>>();
        let f = match f {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("dir_exclude: {}", e),
                ))
            }
        };
        filter.dir_exclude.append(f);
    }
    if let Some(ref f) = options.file_include {
        let f = &mut f
            .iter()
            .map(|s| Pattern::new(s))
            .collect::<Result<Vec<_>, glob::PatternError>>();
        let f = match f {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("file_include: {}", e),
                ))
            }
        };
        filter.file_include.append(f);
    }
    if let Some(ref f) = options.file_exclude {
        let f = &mut f
            .iter()
            .map(|s| Pattern::new(s))
            .collect::<Result<Vec<_>, glob::PatternError>>();
        let f = match f {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("file_exclude: {}", e),
                ))
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

pub fn filter_dir(
    root_path_len: usize,
    dir_entry: &jwalk::DirEntry<((), Option<Result<Metadata, Error>>)>,
    filter_ref: &Filter,
) -> bool {
    let mut key = dir_entry.parent_path.to_path_buf();
    key.push(dir_entry.file_name.clone().into_string().unwrap());
    let key = key
        .to_str()
        .unwrap()
        .get(root_path_len..)
        .unwrap_or("")
        .to_string();
    if filter_direntry(&key, &filter_ref.dir_exclude, filter_ref.options, false) 
        || !filter_direntry(&key, &filter_ref.dir_include, filter_ref.options, true) {
        return false;
    }
    true
}

pub fn filter_children(
    children: &mut Vec<
        Result<jwalk::DirEntry<((), Option<Result<Metadata, Error>>)>, jwalk::Error>,
    >,
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
                        let key = dir_entry.file_name.to_str().unwrap();
                        if filter_direntry(key, &filter_ref.file_exclude, options, false)
                            || !filter_direntry(key, &filter_ref.file_include, options, true) {
                            return false;
                        }
                    }
                    true
                })
                .unwrap_or(false)
        });
    }
}
