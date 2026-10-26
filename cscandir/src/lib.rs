use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::Arc;

use scandir::{Count, ReturnType, Scandir, ScandirResult};

const CSCANDIR_OK: i32 = 0;
const CSCANDIR_ERR_INVALID_ARGUMENT: i32 = 1;
const CSCANDIR_ERR_INVALID_UTF8: i32 = 2;
const CSCANDIR_ERR_SCAN: i32 = 3;
const CSCANDIR_ERR_NUL_BYTE: i32 = 4;
const CSCANDIR_ERR_PANIC: i32 = 5;

struct FfiError(i32, String);

impl From<std::io::Error> for FfiError {
    fn from(error: std::io::Error) -> Self {
        let code = if error.kind() == std::io::ErrorKind::InvalidInput {
            CSCANDIR_ERR_INVALID_ARGUMENT
        } else {
            CSCANDIR_ERR_SCAN
        };
        Self(code, error.to_string())
    }
}

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
#[derive(Default)]
pub struct CScandirEntryList {
    pub entries: *mut CScandirEntry,
    pub len: usize,
    pub capacity: usize,
}

#[repr(C)]
#[derive(Default)]
pub struct CScandirStringList {
    pub items: *mut *mut c_char,
    pub len: usize,
    pub capacity: usize,
}

#[repr(C)]
#[derive(Default)]
pub struct CScandirError {
    pub code: i32,
    pub message: *mut c_char,
}

#[repr(C)]
#[derive(Default)]
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

// Keep ownership inside Rust until all conversions succeed. These destructors
// also roll back partially constructed outputs on errors or unwinding.
impl Drop for CScandirEntry {
    fn drop(&mut self) {
        if !self.path.is_null() {
            // SAFETY: paths are created exclusively by CString::into_raw.
            drop(unsafe { CString::from_raw(self.path) });
        }
    }
}

impl Drop for CScandirEntryList {
    fn drop(&mut self) {
        if !self.entries.is_null() {
            // SAFETY: these are the unchanged parts of a Vec allocated here.
            drop(unsafe { Vec::from_raw_parts(self.entries, self.len, self.capacity) });
        }
    }
}

impl Drop for CScandirStringList {
    fn drop(&mut self) {
        if !self.items.is_null() {
            // SAFETY: these are the unchanged parts of a Vec allocated here.
            let items = unsafe { Vec::from_raw_parts(self.items, self.len, self.capacity) };
            for item in items {
                // SAFETY: every item was produced by CString::into_raw.
                drop(unsafe { CString::from_raw(item) });
            }
        }
    }
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

fn map_return_type(value: u32) -> Result<ReturnType, FfiError> {
    match value {
        0 => Ok(ReturnType::Base),
        1 => Ok(ReturnType::Ext),
        _ => Err(FfiError(
            CSCANDIR_ERR_INVALID_ARGUMENT,
            "Invalid return_type value. Expected 0 or 1.".to_string(),
        )),
    }
}

unsafe fn read_string_array(
    ptrs: *const *const c_char,
    len: usize,
) -> Result<Option<Vec<String>>, FfiError> {
    if len == 0 {
        return Ok(None);
    }
    if ptrs.is_null() || len > isize::MAX as usize / size_of::<*const c_char>() {
        return Err(FfiError(
            CSCANDIR_ERR_INVALID_ARGUMENT,
            "Invalid filter pointer or length".to_string(),
        ));
    }

    let mut out = Vec::with_capacity(len);
    for idx in 0..len {
        // SAFETY: Caller provides a valid pointer array with len elements.
        let p = unsafe { *ptrs.add(idx) };
        if p.is_null() {
            return Err(FfiError(
                CSCANDIR_ERR_INVALID_ARGUMENT,
                format!("String pointer at index {idx} is null"),
            ));
        }
        // SAFETY: Caller promises this is a valid NUL-terminated C string.
        let s = unsafe { CStr::from_ptr(p) };
        let s = s.to_str().map_err(|_| {
            FfiError(
                CSCANDIR_ERR_INVALID_UTF8,
                format!("String at index {idx} is not valid UTF-8"),
            )
        })?;
        out.push(s.to_string());
    }

    Ok(Some(out))
}

unsafe fn clear_entry_list(out_entries: *mut CScandirEntryList) {
    if out_entries.is_null() {
        return;
    }
    // SAFETY: caller supplies writable storage; do not read uninitialized fields.
    unsafe { out_entries.write(CScandirEntryList::default()) };
}

unsafe fn clear_string_list(out_errors: *mut CScandirStringList) {
    if out_errors.is_null() {
        return;
    }
    // SAFETY: caller supplies writable storage; do not read uninitialized fields.
    unsafe { out_errors.write(CScandirStringList::default()) };
}

unsafe fn set_error(out_error: *mut CScandirError, code: i32, message: &str) {
    if out_error.is_null() {
        return;
    }

    let message = match CString::new(message) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    };
    // SAFETY: caller supplies writable storage; outputs are write-only.
    unsafe { out_error.write(CScandirError { code, message }) };
}

unsafe fn clear_error(out_error: *mut CScandirError) {
    if out_error.is_null() {
        return;
    }

    // SAFETY: caller supplies writable storage; do not read uninitialized fields.
    unsafe { out_error.write(CScandirError::default()) };
}

fn entry_to_ffi(entry: ScandirResult) -> Result<CScandirEntry, FfiError> {
    let mut out = CScandirEntry {
        path: ptr::null_mut(),
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

    // Move the existing path buffer into CString rather than copying each path.
    let path = match entry {
        ScandirResult::DirEntry(e) => e.path,
        ScandirResult::DirEntryExt(e) => e.path,
        ScandirResult::Error(_) => unreachable!("errors are separated before conversion"),
    };
    out.path = c_string(path)?.into_raw();

    Ok(out)
}

fn entries_to_ffi(entries: Vec<ScandirResult>) -> Result<CScandirEntryList, FfiError> {
    let mut out = Vec::<CScandirEntry>::with_capacity(entries.len());
    for entry in entries {
        out.push(entry_to_ffi(entry)?);
    }
    if out.is_empty() {
        return Ok(CScandirEntryList::default());
    }

    let list = CScandirEntryList {
        entries: out.as_mut_ptr(),
        len: out.len(),
        capacity: out.capacity(),
    };
    std::mem::forget(out);
    Ok(list)
}

fn c_string(value: String) -> Result<CString, FfiError> {
    CString::new(value).map_err(|_| {
        FfiError(
            CSCANDIR_ERR_NUL_BYTE,
            "Output contains a NUL byte".to_string(),
        )
    })
}

fn strings_to_ffi(items: Vec<String>) -> Result<CScandirStringList, FfiError> {
    // CString keeps each allocation owned if a later conversion fails.
    let mut strings = Vec::with_capacity(items.len());
    for item in items {
        strings.push(c_string(item)?);
    }
    if strings.is_empty() {
        return Ok(CScandirStringList::default());
    }
    let mut out: Vec<*mut c_char> = strings.into_iter().map(CString::into_raw).collect();

    let list = CScandirStringList {
        items: out.as_mut_ptr(),
        len: out.len(),
        capacity: out.capacity(),
    };
    std::mem::forget(out);
    Ok(list)
}

unsafe fn apply_scandir_options(
    scanner: Scandir,
    options: &CScandirOptions,
) -> Result<Scandir, FfiError> {
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

unsafe fn apply_count_options(
    counter: Count,
    options: &CScandirOptions,
) -> Result<Count, FfiError> {
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
        options.write(default_options());
    }
}

mod api;
pub use api::{cscandir_collect, cscandir_count};

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
    drop(std::mem::take(list));
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
    drop(std::mem::take(list));
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
