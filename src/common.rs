#[cfg(unix)]
use expanduser::expanduser;

use glob::{MatchOptions, Pattern};
use jwalk::WalkDirGeneric;

use crate::def::*;

#[cfg(unix)]
pub fn expand_path(path: &str) -> String {
    let path = expanduser(path).unwrap();
    path.to_string_lossy().into_owned()
}

pub fn create_filter(
    dir_include: Option<Vec<String>>,
    dir_exclude: Option<Vec<String>>,
    file_include: Option<Vec<String>>,
    file_exclude: Option<Vec<String>>,
    case_sensitive: Option<bool>,
) -> Result<Option<Filter>, glob::PatternError> {
    let mut filter = Filter {
        dir_include: Vec::new(),
        dir_exclude: Vec::new(),
        file_include: Vec::new(),
        file_exclude: Vec::new(),
        options: match case_sensitive.unwrap_or(false) {
            true => None,
            false => Some(MatchOptions {
                case_sensitive: false,
                ..MatchOptions::new()
            }),
        },
    };
    match dir_include {
        Some(f) => {
            let f = &mut f
                .iter()
                .map(|s| Pattern::new(s))
                .collect::<Result<Vec<_>, glob::PatternError>>()?;
            filter.dir_include.append(f);
        }
        None => {}
    }
    match dir_exclude {
        Some(f) => {
            let f = &mut f
                .iter()
                .map(|s| Pattern::new(s))
                .collect::<Result<Vec<_>, glob::PatternError>>()?;
            filter.dir_exclude.append(f);
        }
        None => {}
    }
    match file_include {
        Some(f) => {
            let f = &mut f
                .iter()
                .map(|s| Pattern::new(s))
                .collect::<Result<Vec<_>, glob::PatternError>>()?;
            filter.file_include.append(f);
        }
        None => {}
    }
    match file_exclude {
        Some(f) => {
            let f = &mut f
                .iter()
                .map(|s| Pattern::new(s))
                .collect::<Result<Vec<_>, glob::PatternError>>()?;
            filter.file_exclude.append(f);
        }
        None => {}
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

fn filter_direntry(
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
                if f.as_str().ends_with("**") && !key.ends_with("/") {
                    // Workaround: glob currently has problems with "foo/**"
                    let mut key = String::from(key);
                    key.push_str("/");
                    if f.matches_with(&key, options) {
                        return true;
                    }
                }
                if f.matches_with(&key, options) {
                    return true;
                }
            }
        }
        None => {
            for f in filter {
                if f.as_str().ends_with("**") && !key.ends_with("/") {
                    // Workaround: glob currently has problems with "foo/**"
                    let mut key = String::from(key);
                    key.push_str("/");
                    if f.matches(&key) {
                        return true;
                    }
                }
                if f.matches(&key) {
                    return true;
                }
            }
        }
    }
    false
}

fn filter_dir(
    root_path_len: usize,
    dir_entry: &jwalk::core::dir_entry::DirEntry<()>,
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
    if filter_direntry(&key, &filter_ref.dir_exclude, filter_ref.options, false) {
        return false;
    } else if !filter_direntry(&key, &filter_ref.dir_include, filter_ref.options, true) {
        return false;
    }
    true
}

pub fn walk(
    root_path: &str,
    sorted: bool,
    skip_hidden: bool,
    mut max_depth: usize,
    filter: Option<Filter>,
    return_type: u8,
) -> WalkDirGeneric<()> {
    let root_path_len = root_path.len() + 1;
    if max_depth == 0 {
        max_depth = ::std::usize::MAX;
    };
    if filter.is_none() {
        return WalkDirGeneric::new(root_path)
            .skip_hidden(skip_hidden)
            .sort(sorted)
            .preload_metadata(return_type > RETURN_TYPE_FAST)
            .preload_metadata_ext(return_type > RETURN_TYPE_BASE)
            .max_depth(max_depth);
    }
    WalkDirGeneric::new(root_path)
        .skip_hidden(skip_hidden)
        .sort(sorted)
        .preload_metadata(return_type > RETURN_TYPE_FAST)
        .preload_metadata_ext(return_type > RETURN_TYPE_BASE)
        .max_depth(max_depth)
        .process_entries(move |_parent_client_state, children| {
            // Custom filter
            children.retain(|dir_entry_result| {
                dir_entry_result
                    .as_ref()
                    .map(|dir_entry| {
                        let filter_ref = filter.as_ref().unwrap();
                        if dir_entry.file_type_result.as_ref().unwrap().is_dir() {
                            return filter_dir(root_path_len, dir_entry, filter_ref);
                        } else {
                            let options = filter_ref.options;
                            let key = dir_entry.file_name.to_str().unwrap();
                            if filter_direntry(key, &filter_ref.file_exclude, options, false) {
                                return false;
                            } else if !filter_direntry(key, &filter_ref.file_include, options, true)
                            {
                                return false;
                            }
                        }
                        true
                    })
                    .unwrap_or(false)
            });
            // Custom skip
            children.iter_mut().for_each(|dir_entry_result| {
                if filter.is_none() {
                    return ().into();
                }
                if let Ok(dir_entry) = dir_entry_result {
                    if !dir_entry.file_type_result.as_ref().unwrap().is_dir() {
                        return ().into();
                    }
                    if !filter_dir(root_path_len, dir_entry, filter.as_ref().unwrap()) {
                        dir_entry.read_children_path = None;
                    }
                }
            });
        })
}
