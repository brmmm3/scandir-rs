use std::ffi::{CStr, CString, c_char};
use std::ptr;

use scandir::{Count, ReturnType, Scandir, ScandirResult};

const CSCANDIR_OK: i32 = 0;
const CSCANDIR_ERR_INVALID_ARGUMENT: i32 = 1;
const CSCANDIR_ERR_INVALID_UTF8: i32 = 2;
const CSCANDIR_ERR_SCAN: i32 = 3;
const CSCANDIR_ERR_NUL_BYTE: i32 = 4;

#[repr(C)]
pub struct CScandirOptions {
    pub sorted: u8,
    pub skip_hidden: u8,
    pub max_depth: usize,
    pub max_file_cnt: usize,
    pub dir_include: *const *const c_char,
    pub dir_include_len: usize,
    pub dir_exclude: *const *const c_char,
    pub dir_exclude_len: usize,
    pub file_include: *const *const c_char,
    pub file_include_len: usize,
    pub file_exclude: *const *const c_char,
    pub file_exclude_len: usize,
    pub case_sensitive: u8,
    pub follow_links: u8,
    pub return_type: u32,
}

#[repr(C)]
pub struct CScandirEntry {
    pub path: *mut c_char,
    pub is_symlink: u8,
    pub is_dir: u8,
    pub is_file: u8,
    pub ctime: f64,
    pub mtime: f64,
    pub atime: f64,
    pub size: u64,
    pub has_ext: u8,
    pub mode: u32,
    pub ino: u64,
    pub dev: u64,
    pub nlink: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
}

#[repr(C)]
pub struct CScandirEntryList {
    pub entries: *mut CScandirEntry,
    pub len: usize,
    pub capacity: usize,
}

#[repr(C)]
pub struct CScandirStringList {
    pub items: *mut *mut c_char,
    pub len: usize,
    pub capacity: usize,
}

#[repr(C)]
pub struct CScandirError {
    pub code: i32,
    pub message: *mut c_char,
}

#[repr(C)]
pub struct CScandirStatistics {
    pub dirs: i32,
    pub files: i32,
    pub slinks: i32,
    pub hlinks: i32,
    pub devices: i32,
    pub pipes: i32,
    pub size: u64,
    pub usage: u64,
    pub duration: f64,
}

fn default_options() -> CScandirOptions {
    CScandirOptions {
        sorted: 0,
        skip_hidden: 0,
        max_depth: usize::MAX,
        max_file_cnt: usize::MAX,
        dir_include: ptr::null(),
        dir_include_len: 0,
        dir_exclude: ptr::null(),
        dir_exclude_len: 0,
        file_include: ptr::null(),
        file_include_len: 0,
        file_exclude: ptr::null(),
        file_exclude_len: 0,
        case_sensitive: 0,
        follow_links: 0,
        return_type: 0,
    }
}

fn to_bool(value: u8) -> bool {
    value != 0
}

fn map_return_type(value: u32) -> Result<ReturnType, String> {
    match value {
        0 => Ok(ReturnType::Base),
        1 => Ok(ReturnType::Ext),
        _ => Err("Invalid return_type value. Expected 0 or 1.".to_string()),
    }
}

unsafe fn read_string_array(
    ptrs: *const *const c_char,
    len: usize,
) -> Result<Option<Vec<String>>, String> {
    if ptrs.is_null() || len == 0 {
        return Ok(None);
    }

    let mut out = Vec::with_capacity(len);
    for idx in 0..len {
        // SAFETY: Caller provides a valid pointer array with len elements.
        let p = unsafe { *ptrs.add(idx) };
        if p.is_null() {
            return Err(format!("String pointer at index {idx} is null"));
        }
        // SAFETY: Caller promises this is a valid NUL-terminated C string.
        let s = unsafe { CStr::from_ptr(p) };
        let s = s
            .to_str()
            .map_err(|_| format!("String at index {idx} is not valid UTF-8"))?;
        out.push(s.to_string());
    }

    if out.is_empty() {
        Ok(None)
    } else {
        Ok(Some(out))
    }
}

unsafe fn clear_entry_list(out_entries: *mut CScandirEntryList) {
    if out_entries.is_null() {
        return;
    }
    // SAFETY: pointer checked for null.
    let out_entries = unsafe { &mut *out_entries };
    out_entries.entries = ptr::null_mut();
    out_entries.len = 0;
    out_entries.capacity = 0;
}

unsafe fn clear_string_list(out_errors: *mut CScandirStringList) {
    if out_errors.is_null() {
        return;
    }
    // SAFETY: pointer checked for null.
    let out_errors = unsafe { &mut *out_errors };
    out_errors.items = ptr::null_mut();
    out_errors.len = 0;
    out_errors.capacity = 0;
}

unsafe fn set_error(out_error: *mut CScandirError, code: i32, message: &str) {
    if out_error.is_null() {
        return;
    }

    // SAFETY: pointer checked for null.
    let out_error = unsafe { &mut *out_error };
    out_error.code = code;

    if !out_error.message.is_null() {
        // SAFETY: Message was previously allocated by CString::into_raw in this library.
        let _ = unsafe { CString::from_raw(out_error.message) };
    }

    out_error.message = match CString::new(message) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    };
}

unsafe fn clear_error(out_error: *mut CScandirError) {
    if out_error.is_null() {
        return;
    }

    // SAFETY: pointer checked for null.
    let out_error = unsafe { &mut *out_error };
    out_error.code = CSCANDIR_OK;

    if !out_error.message.is_null() {
        // SAFETY: Message was previously allocated by CString::into_raw in this library.
        let _ = unsafe { CString::from_raw(out_error.message) };
    }

    out_error.message = ptr::null_mut();
}

fn entry_to_ffi(entry: &ScandirResult) -> Result<CScandirEntry, i32> {
    let path = CString::new(entry.path().as_str()).map_err(|_| CSCANDIR_ERR_NUL_BYTE)?;

    let mut out = CScandirEntry {
        path: path.into_raw(),
        is_symlink: u8::from(entry.is_symlink()),
        is_dir: u8::from(entry.is_dir()),
        is_file: u8::from(entry.is_file()),
        ctime: entry.ctime(),
        mtime: entry.mtime(),
        atime: entry.atime(),
        size: entry.size(),
        has_ext: 0,
        mode: 0,
        ino: 0,
        dev: 0,
        nlink: 0,
        blksize: 0,
        blocks: 0,
        uid: 0,
        gid: 0,
        rdev: 0,
    };

    if let Some(ext) = entry.ext() {
        out.has_ext = 1;
        out.mode = ext.st_mode;
        out.ino = ext.st_ino;
        out.dev = ext.st_dev;
        out.nlink = ext.st_nlink;
        out.blksize = ext.st_blksize;
        out.blocks = ext.st_blocks;
        out.uid = ext.st_uid;
        out.gid = ext.st_gid;
        out.rdev = ext.st_rdev;
    }

    Ok(out)
}

fn entries_to_ffi(entries: &[ScandirResult]) -> Result<CScandirEntryList, i32> {
    if entries.is_empty() {
        return Ok(CScandirEntryList {
            entries: ptr::null_mut(),
            len: 0,
            capacity: 0,
        });
    }

    let mut out = Vec::<CScandirEntry>::with_capacity(entries.len());
    for entry in entries {
        match entry_to_ffi(entry) {
            Ok(v) => out.push(v),
            Err(code) => {
                for item in out {
                    if !item.path.is_null() {
                        // SAFETY: Path was allocated using CString::into_raw in this library.
                        let _ = unsafe { CString::from_raw(item.path) };
                    }
                }
                return Err(code);
            }
        }
    }

    let list = CScandirEntryList {
        entries: out.as_mut_ptr(),
        len: out.len(),
        capacity: out.capacity(),
    };
    std::mem::forget(out);
    Ok(list)
}

fn strings_to_ffi(items: &[String]) -> Result<CScandirStringList, i32> {
    if items.is_empty() {
        return Ok(CScandirStringList {
            items: ptr::null_mut(),
            len: 0,
            capacity: 0,
        });
    }

    let mut out: Vec<*mut c_char> = Vec::with_capacity(items.len());
    for item in items {
        match CString::new(item.as_str()) {
            Ok(s) => out.push(s.into_raw()),
            Err(_) => {
                for p in out {
                    if !p.is_null() {
                        // SAFETY: Pointer was allocated using CString::into_raw in this library.
                        let _ = unsafe { CString::from_raw(p) };
                    }
                }
                return Err(CSCANDIR_ERR_NUL_BYTE);
            }
        }
    }

    let list = CScandirStringList {
        items: out.as_mut_ptr(),
        len: out.len(),
        capacity: out.capacity(),
    };
    std::mem::forget(out);
    Ok(list)
}

fn apply_scandir_options(scanner: Scandir, options: &CScandirOptions) -> Result<Scandir, String> {
    let return_type = map_return_type(options.return_type)?;

    // SAFETY: Pointers are read-only and lengths are controlled by the caller.
    let dir_include = unsafe { read_string_array(options.dir_include, options.dir_include_len)? };
    // SAFETY: Pointers are read-only and lengths are controlled by the caller.
    let dir_exclude = unsafe { read_string_array(options.dir_exclude, options.dir_exclude_len)? };
    // SAFETY: Pointers are read-only and lengths are controlled by the caller.
    let file_include =
        unsafe { read_string_array(options.file_include, options.file_include_len)? };
    // SAFETY: Pointers are read-only and lengths are controlled by the caller.
    let file_exclude =
        unsafe { read_string_array(options.file_exclude, options.file_exclude_len)? };

    Ok(scanner
        .sorted(to_bool(options.sorted))
        .skip_hidden(to_bool(options.skip_hidden))
        .max_depth(options.max_depth)
        .max_file_cnt(options.max_file_cnt)
        .dir_include(dir_include)
        .dir_exclude(dir_exclude)
        .file_include(file_include)
        .file_exclude(file_exclude)
        .case_sensitive(to_bool(options.case_sensitive))
        .follow_links(to_bool(options.follow_links))
        .return_type(return_type))
}

fn apply_count_options(counter: Count, options: &CScandirOptions) -> Result<Count, String> {
    let return_type = map_return_type(options.return_type)?;

    // SAFETY: Pointers are read-only and lengths are controlled by the caller.
    let dir_include = unsafe { read_string_array(options.dir_include, options.dir_include_len)? };
    // SAFETY: Pointers are read-only and lengths are controlled by the caller.
    let dir_exclude = unsafe { read_string_array(options.dir_exclude, options.dir_exclude_len)? };
    // SAFETY: Pointers are read-only and lengths are controlled by the caller.
    let file_include =
        unsafe { read_string_array(options.file_include, options.file_include_len)? };
    // SAFETY: Pointers are read-only and lengths are controlled by the caller.
    let file_exclude =
        unsafe { read_string_array(options.file_exclude, options.file_exclude_len)? };

    Ok(counter
        .skip_hidden(to_bool(options.skip_hidden))
        .max_depth(options.max_depth)
        .max_file_cnt(options.max_file_cnt)
        .dir_include(dir_include)
        .dir_exclude(dir_exclude)
        .file_include(file_include)
        .file_exclude(file_exclude)
        .case_sensitive(to_bool(options.case_sensitive))
        .follow_links(to_bool(options.follow_links))
        .extended(return_type == ReturnType::Ext))
}

/// # Safety
///
/// This function uses unsafe code and raw pointers to interface with C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cscandir_options_init(options: *mut CScandirOptions) {
    if options.is_null() {
        return;
    }
    // SAFETY: pointer checked for null.
    unsafe {
        *options = default_options();
    }
}

/// # Safety
///
/// This function uses unsafe code and raw pointers to interface with C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cscandir_collect(
    root_path: *const c_char,
    options: *const CScandirOptions,
    out_entries: *mut CScandirEntryList,
    out_errors: *mut CScandirStringList,
    out_error: *mut CScandirError,
) -> i32 {
    // SAFETY: Output pointers are optional.
    unsafe {
        clear_entry_list(out_entries);
        clear_string_list(out_errors);
        clear_error(out_error);
    }

    if root_path.is_null() {
        // SAFETY: out_error is optional.
        unsafe {
            set_error(
                out_error,
                CSCANDIR_ERR_INVALID_ARGUMENT,
                "root_path is null",
            )
        };
        return CSCANDIR_ERR_INVALID_ARGUMENT;
    }

    // SAFETY: root_path checked for null and expected to point to a valid C string.
    let root_path_str = match unsafe { CStr::from_ptr(root_path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            // SAFETY: out_error is optional.
            unsafe {
                set_error(
                    out_error,
                    CSCANDIR_ERR_INVALID_UTF8,
                    "root_path is not valid UTF-8",
                )
            };
            return CSCANDIR_ERR_INVALID_UTF8;
        }
    };

    let default_opts = default_options();
    let opt_ref = if options.is_null() {
        &default_opts
    } else {
        // SAFETY: options pointer checked for null.
        unsafe { &*options }
    };

    let scanner = match Scandir::new(root_path_str, Some(true)) {
        Ok(v) => v,
        Err(e) => {
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, CSCANDIR_ERR_SCAN, &e.to_string()) };
            return CSCANDIR_ERR_SCAN;
        }
    };

    let mut scanner = match apply_scandir_options(scanner, opt_ref) {
        Ok(v) => v,
        Err(e) => {
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, CSCANDIR_ERR_INVALID_ARGUMENT, &e) };
            return CSCANDIR_ERR_INVALID_ARGUMENT;
        }
    };

    let results = match scanner.collect() {
        Ok(v) => v,
        Err(e) => {
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, CSCANDIR_ERR_SCAN, &e.to_string()) };
            return CSCANDIR_ERR_SCAN;
        }
    };

    let mut entries = match entries_to_ffi(&results.results) {
        Ok(v) => v,
        Err(code) => {
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, code, "Failed to convert entry to C string") };
            return code;
        }
    };

    let errors_vec: Vec<String> = results
        .errors
        .iter()
        .map(|(path, err)| {
            if path.is_empty() {
                err.clone()
            } else {
                format!("{path}: {err}")
            }
        })
        .collect();

    let mut errors = match strings_to_ffi(&errors_vec) {
        Ok(v) => v,
        Err(code) => {
            // SAFETY: entries list came from this library.
            unsafe { cscandir_free_entry_list(&mut entries as *mut CScandirEntryList) };
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, code, "Failed to convert error text to C string") };
            return code;
        }
    };

    if !out_entries.is_null() {
        // SAFETY: pointer checked for null.
        unsafe { *out_entries = entries };
    } else {
        // SAFETY: entries list came from this library.
        unsafe { cscandir_free_entry_list(&mut entries as *mut CScandirEntryList) };
    }

    if !out_errors.is_null() {
        // SAFETY: pointer checked for null.
        unsafe { *out_errors = errors };
    } else {
        // SAFETY: errors list came from this library.
        unsafe { cscandir_free_string_list(&mut errors as *mut CScandirStringList) };
    }

    CSCANDIR_OK
}

/// # Safety
///
/// This function uses unsafe code and raw pointers to interface with C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cscandir_count(
    root_path: *const c_char,
    options: *const CScandirOptions,
    out_stats: *mut CScandirStatistics,
    out_errors: *mut CScandirStringList,
    out_error: *mut CScandirError,
) -> i32 {
    // SAFETY: Output pointers are optional.
    unsafe {
        clear_string_list(out_errors);
        clear_error(out_error);
    }

    if !out_stats.is_null() {
        // SAFETY: out_stats pointer checked for null.
        unsafe {
            *out_stats = CScandirStatistics {
                dirs: 0,
                files: 0,
                slinks: 0,
                hlinks: 0,
                devices: 0,
                pipes: 0,
                size: 0,
                usage: 0,
                duration: 0.0,
            };
        }
    }

    if root_path.is_null() {
        // SAFETY: out_error is optional.
        unsafe {
            set_error(
                out_error,
                CSCANDIR_ERR_INVALID_ARGUMENT,
                "root_path is null",
            )
        };
        return CSCANDIR_ERR_INVALID_ARGUMENT;
    }

    // SAFETY: root_path checked for null and expected to point to a valid C string.
    let root_path_str = match unsafe { CStr::from_ptr(root_path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            // SAFETY: out_error is optional.
            unsafe {
                set_error(
                    out_error,
                    CSCANDIR_ERR_INVALID_UTF8,
                    "root_path is not valid UTF-8",
                )
            };
            return CSCANDIR_ERR_INVALID_UTF8;
        }
    };

    let default_opts = default_options();
    let opt_ref = if options.is_null() {
        &default_opts
    } else {
        // SAFETY: options pointer checked for null.
        unsafe { &*options }
    };

    let counter = match Count::new(root_path_str) {
        Ok(v) => v,
        Err(e) => {
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, CSCANDIR_ERR_SCAN, &e.to_string()) };
            return CSCANDIR_ERR_SCAN;
        }
    };

    let mut counter = match apply_count_options(counter, opt_ref) {
        Ok(v) => v,
        Err(e) => {
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, CSCANDIR_ERR_INVALID_ARGUMENT, &e) };
            return CSCANDIR_ERR_INVALID_ARGUMENT;
        }
    };

    let stats = match counter.collect() {
        Ok(v) => v,
        Err(e) => {
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, CSCANDIR_ERR_SCAN, &e.to_string()) };
            return CSCANDIR_ERR_SCAN;
        }
    };

    if !out_stats.is_null() {
        // SAFETY: out_stats pointer checked for null.
        unsafe {
            *out_stats = CScandirStatistics {
                dirs: stats.dirs,
                files: stats.files,
                slinks: stats.slinks,
                hlinks: stats.hlinks,
                devices: stats.devices,
                pipes: stats.pipes,
                size: stats.size,
                usage: stats.usage,
                duration: stats.duration,
            };
        }
    }

    let mut errors = match strings_to_ffi(&stats.errors) {
        Ok(v) => v,
        Err(code) => {
            // SAFETY: out_error is optional.
            unsafe { set_error(out_error, code, "Failed to convert error text to C string") };
            return code;
        }
    };

    if !out_errors.is_null() {
        // SAFETY: pointer checked for null.
        unsafe { *out_errors = errors };
    } else {
        // SAFETY: errors list came from this library.
        unsafe { cscandir_free_string_list(&mut errors as *mut CScandirStringList) };
    }

    CSCANDIR_OK
}

/// # Safety
///
/// This function uses unsafe code and raw pointers to interface with C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cscandir_free_entry_list(list: *mut CScandirEntryList) {
    if list.is_null() {
        return;
    }

    // SAFETY: pointer checked for null.
    let list = unsafe { &mut *list };
    if !list.entries.is_null() {
        // SAFETY: parts were allocated by this library.
        let entries = unsafe { Vec::from_raw_parts(list.entries, list.len, list.capacity) };
        for entry in entries {
            if !entry.path.is_null() {
                // SAFETY: path was allocated by CString::into_raw in this library.
                let _ = unsafe { CString::from_raw(entry.path) };
            }
        }
    }

    list.entries = ptr::null_mut();
    list.len = 0;
    list.capacity = 0;
}

/// # Safety
///
/// This function uses unsafe code and raw pointers to interface with C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cscandir_free_string_list(list: *mut CScandirStringList) {
    if list.is_null() {
        return;
    }

    // SAFETY: pointer checked for null.
    let list = unsafe { &mut *list };
    if !list.items.is_null() {
        // SAFETY: parts were allocated by this library.
        let items = unsafe { Vec::from_raw_parts(list.items, list.len, list.capacity) };
        for item in items {
            if !item.is_null() {
                // SAFETY: string was allocated by CString::into_raw in this library.
                let _ = unsafe { CString::from_raw(item) };
            }
        }
    }

    list.items = ptr::null_mut();
    list.len = 0;
    list.capacity = 0;
}

/// # Safety
///
/// This function uses unsafe code and raw pointers to interface with C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cscandir_free_error(error: *mut CScandirError) {
    if error.is_null() {
        return;
    }

    // SAFETY: pointer checked for null.
    let error = unsafe { &mut *error };
    if !error.message.is_null() {
        // SAFETY: message was allocated by CString::into_raw in this library.
        let _ = unsafe { CString::from_raw(error.message) };
    }

    error.message = ptr::null_mut();
    error.code = CSCANDIR_OK;
}
