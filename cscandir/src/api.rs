use super::*;

unsafe fn run_ffi(
    out_error: *mut CScandirError,
    operation: impl FnOnce() -> Result<(), FfiError>,
) -> i32 {
    // SAFETY: the caller supplies optional writable output storage.
    unsafe { clear_error(out_error) };
    let error = match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => return CSCANDIR_OK,
        Ok(Err(error)) => error,
        Err(_) => FfiError(CSCANDIR_ERR_PANIC, "Rust panic during scan".to_string()),
    };
    // SAFETY: this output was initialized above and has not been published yet.
    unsafe { set_error(out_error, error.0, &error.1) };
    error.0
}

unsafe fn root_from_ptr<'a>(root: *const c_char) -> Result<&'a str, FfiError> {
    if root.is_null() {
        return Err(FfiError(
            CSCANDIR_ERR_INVALID_ARGUMENT,
            "root_path is null".to_string(),
        ));
    }
    // SAFETY: caller guarantees a readable, NUL-terminated C string.
    unsafe { CStr::from_ptr(root) }.to_str().map_err(|_| {
        FfiError(
            CSCANDIR_ERR_INVALID_UTF8,
            "root_path is not valid UTF-8".to_string(),
        )
    })
}

fn collect_outputs(
    mut results: scandir::ScandirResults,
    want_entries: bool,
    want_errors: bool,
) -> Result<(CScandirEntryList, CScandirStringList), FfiError> {
    // The core normally separates errors, but handle embedded error variants too.
    let has_embedded_errors = results.results.iter().any(|e| e.error().is_some());
    if has_embedded_errors {
        let mut entries = Vec::with_capacity(results.results.len());
        for result in results.results {
            match result {
                ScandirResult::Error(error) => results.errors.push(error),
                entry if want_entries => entries.push(entry),
                _ => {}
            }
        }
        results.results = entries;
    }
    let entries = if want_entries {
        entries_to_ffi(results.results)?
    } else {
        CScandirEntryList::default()
    };
    let errors = if want_errors {
        strings_to_ffi(
            results
                .errors
                .into_iter()
                .map(|(path, error)| {
                    if path.is_empty() {
                        error
                    } else {
                        format!("{path}: {error}")
                    }
                })
                .collect(),
        )?
    } else {
        CScandirStringList::default()
    };
    Ok((entries, errors))
}

/// Collect directory entries without allowing Rust unwinding across the C ABI.
///
/// # Safety
/// Inputs must be readable for the duration of the call, strings NUL-terminated,
/// and non-null outputs writable, aligned and non-overlapping. Free previous
/// outputs before reusing their storage. See the public header for ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cscandir_collect(
    root_path: *const c_char,
    options: *const CScandirOptions,
    out_entries: *mut CScandirEntryList,
    out_errors: *mut CScandirStringList,
    out_error: *mut CScandirError,
) -> i32 {
    // SAFETY: outputs are optional writable storage, possibly uninitialized.
    unsafe {
        clear_entry_list(out_entries);
        clear_string_list(out_errors);
        run_ffi(out_error, || {
            let root = root_from_ptr(root_path)?;
            let defaults = default_options();
            let options = options.as_ref().unwrap_or(&defaults);
            let scanner = Scandir::new(root, Some(false))?;
            let mut scanner = apply_scandir_options(scanner, options)?;
            let results = scanner.collect()?;
            // collect() also retains an Arc. Drop the scanner before unwrapping
            // so that path strings can be moved rather than cloned.
            drop(scanner);
            let (entries, errors) = collect_outputs(
                Arc::unwrap_or_clone(results),
                !out_entries.is_null(),
                !out_errors.is_null(),
            )?;
            if !out_entries.is_null() {
                out_entries.write(entries);
            }
            if !out_errors.is_null() {
                out_errors.write(errors);
            }
            Ok(())
        })
    }
}

/// Collect aggregate directory statistics.
///
/// # Safety
/// Same pointer, lifetime and output ownership requirements as `cscandir_collect`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cscandir_count(
    root_path: *const c_char,
    options: *const CScandirOptions,
    out_stats: *mut CScandirStatistics,
    out_errors: *mut CScandirStringList,
    out_error: *mut CScandirError,
) -> i32 {
    // SAFETY: outputs are optional writable storage, possibly uninitialized.
    unsafe {
        clear_string_list(out_errors);
        if !out_stats.is_null() {
            out_stats.write(CScandirStatistics::default());
        }
        run_ffi(out_error, || {
            let root = root_from_ptr(root_path)?;
            let defaults = default_options();
            let options = options.as_ref().unwrap_or(&defaults);
            let counter = Count::new(root)?;
            let mut counter = apply_count_options(counter, options)?;
            let stats = counter.collect()?;
            let errors = if out_errors.is_null() {
                CScandirStringList::default()
            } else {
                strings_to_ffi(stats.errors)?
            };
            if !out_stats.is_null() {
                out_stats.write(CScandirStatistics {
                    dirs: stats.dirs,
                    files: stats.files,
                    slinks: stats.slinks,
                    hlinks: stats.hlinks,
                    devices: stats.devices,
                    pipes: stats.pipes,
                    size: stats.size,
                    usage: stats.usage,
                    duration: stats.duration,
                });
            }
            if !out_errors.is_null() {
                out_errors.write(errors);
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_is_contained() {
        let mut error = CScandirError::default();
        let code = unsafe { run_ffi(&mut error, || panic!("test panic")) };
        assert_eq!(code, CSCANDIR_ERR_PANIC);
        assert_eq!(error.code, code);
        unsafe { cscandir_free_error(&mut error) };
    }

    #[test]
    fn conversion_failures_and_embedded_errors() {
        let mut results = scandir::ScandirResults::new();
        results
            .results
            .push(ScandirResult::Error(("path".into(), "failure".into())));
        let (entries, errors) = collect_outputs(results, true, true).unwrap_or_else(|_| panic!());
        assert_eq!(entries.len, 0);
        assert_eq!(errors.len, 1);
        assert!(strings_to_ffi(vec!["valid".into(), "bad\0string".into()]).is_err());
        assert!(
            entries_to_ffi(vec![
                ScandirResult::DirEntry(scandir::DirEntry {
                    path: "valid".into(),
                    ..Default::default()
                }),
                ScandirResult::DirEntry(scandir::DirEntry {
                    path: "bad\0path".into(),
                    ..Default::default()
                }),
            ])
            .is_err()
        );
    }
}
