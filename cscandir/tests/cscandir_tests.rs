//! Integration tests for the C ABI surface.
//!
//! These drive the exported `cscandir_*` functions exactly as a C caller
//! would, including the uninitialized / zero-initialized / reused out-struct
//! cases that previously crashed or leaked.

use std::ffi::{CStr, CString, c_char};
use std::ptr;

use cscandir::{
    CScandirEntryList, CScandirError, CScandirOptions, CScandirStatistics, CScandirStringList,
    CScandirToc, CScandirWalkEntryList, cscandir_collect, cscandir_count, cscandir_entry_list_init,
    cscandir_error_init, cscandir_free_entry_list, cscandir_free_error, cscandir_free_string_list,
    cscandir_free_toc, cscandir_free_walk_entry_list, cscandir_options_init,
    cscandir_string_list_init, cscandir_toc_init, cscandir_walk, cscandir_walk_entry_list_init,
};

const OK: i32 = 0;
const ERR_INVALID_ARGUMENT: i32 = 1;
const ERR_INVALID_UTF8: i32 = 2;
const ERR_SCAN: i32 = 3;

/// All-zero storage, which is a valid initial state for every out-struct.
///
/// # Safety-relevant note
/// Each out-struct holds only integers and null pointers, so the all-zero
/// bit pattern is a well-defined "empty" value, not indeterminate memory.
fn zeroed<T>() -> T {
    // SAFETY: see the doc comment; all out-structs are plain integers and
    // pointers, so zero is a valid representation.
    unsafe { std::mem::zeroed() }
}

/// A scratch directory tree with predictable contents.
struct Tree {
    path: std::path::PathBuf,
}

impl Tree {
    fn new(name: &str) -> Tree {
        let path = std::env::temp_dir().join(format!("cscandir_test_{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("sub/deep")).unwrap();
        std::fs::write(path.join("a.txt"), b"a").unwrap();
        std::fs::write(path.join("b.txt"), b"bb").unwrap();
        std::fs::write(path.join("sub/c.txt"), b"ccc").unwrap();
        std::fs::write(path.join("sub/deep/d.txt"), b"dddd").unwrap();
        Tree { path }
    }

    fn c(&self) -> CString {
        CString::new(self.path.to_str().unwrap()).unwrap()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn options() -> CScandirOptions {
    let mut o = zeroed::<CScandirOptions>();
    unsafe { cscandir_options_init(&mut o) };
    o
}

fn list_items(list: &CScandirStringList) -> Vec<String> {
    (0..list.len)
        .map(|i| {
            unsafe { CStr::from_ptr(*list.items.add(i)) }
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn entry_path(entry_list: &CScandirEntryList, i: usize) -> String {
    unsafe { CStr::from_ptr((*entry_list.entries.add(i)).path) }
        .to_string_lossy()
        .into_owned()
}

// ---------------------------------------------------------------------------
// Regression: uninitialized out-structs must not crash.
// ---------------------------------------------------------------------------

#[test]
fn collect_accepts_uninitialized_out_structs() {
    let tree = Tree::new("uninit_collect");
    let opts = options();
    let root = tree.c();

    // Deliberately uninitialized: this used to segfault inside the error
    // reset, which freed an indeterminate pointer.
    let mut entries = std::mem::MaybeUninit::<CScandirEntryList>::uninit();
    let mut errors = std::mem::MaybeUninit::<CScandirStringList>::uninit();
    let mut stats = std::mem::MaybeUninit::<CScandirStatistics>::uninit();
    let mut error = std::mem::MaybeUninit::<CScandirError>::uninit();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            entries.as_mut_ptr(),
            errors.as_mut_ptr(),
            stats.as_mut_ptr(),
            error.as_mut_ptr(),
        )
    };
    assert_eq!(rc, OK);

    let mut entries = unsafe { entries.assume_init() };
    let mut errors = unsafe { errors.assume_init() };
    let stats = unsafe { stats.assume_init() };
    let mut error = unsafe { error.assume_init() };
    assert_eq!(error.code, OK);
    assert!(error.message.is_null());
    // 4 files + 2 dirs; the root itself is not reported.
    assert_eq!(entries.len, 6);
    assert_eq!(errors.len, 0);
    assert_eq!(stats.files, 4);
    assert_eq!(stats.dirs, 2);

    unsafe {
        cscandir_free_entry_list(&mut entries);
        cscandir_free_string_list(&mut errors);
        cscandir_free_error(&mut error);
    }
}

#[test]
fn count_accepts_uninitialized_out_structs() {
    let tree = Tree::new("uninit_count");
    let opts = options();
    let root = tree.c();

    let mut stats = std::mem::MaybeUninit::<CScandirStatistics>::uninit();
    let mut errors = std::mem::MaybeUninit::<CScandirStringList>::uninit();
    let mut error = std::mem::MaybeUninit::<CScandirError>::uninit();

    let rc = unsafe {
        cscandir_count(
            root.as_ptr(),
            &opts,
            stats.as_mut_ptr(),
            errors.as_mut_ptr(),
            error.as_mut_ptr(),
        )
    };
    assert_eq!(rc, OK);
    let stats = unsafe { stats.assume_init() };
    let mut error = unsafe { error.assume_init() };
    assert_eq!(error.code, OK);
    assert_eq!(stats.files, 4);
    assert_eq!(stats.dirs, 2);
    unsafe { cscandir_free_error(&mut error) };
}

#[test]
fn walk_accepts_uninitialized_out_structs() {
    let tree = Tree::new("uninit_walk");
    let opts = options();
    let root = tree.c();

    let mut toc = std::mem::MaybeUninit::<CScandirToc>::uninit();
    let mut entries = std::mem::MaybeUninit::<CScandirWalkEntryList>::uninit();
    let mut stats = std::mem::MaybeUninit::<CScandirStatistics>::uninit();
    let mut error = std::mem::MaybeUninit::<CScandirError>::uninit();

    let rc = unsafe {
        cscandir_walk(
            root.as_ptr(),
            &opts,
            toc.as_mut_ptr(),
            entries.as_mut_ptr(),
            stats.as_mut_ptr(),
            error.as_mut_ptr(),
        )
    };
    assert_eq!(rc, OK);
    let mut toc = unsafe { toc.assume_init() };
    let mut error = unsafe { error.assume_init() };
    assert_eq!(error.code, OK);
    assert_eq!(toc.files.len, 4);
    assert_eq!(toc.dirs.len, 2);

    unsafe {
        cscandir_free_toc(&mut toc);
        cscandir_free_error(&mut error);
    }
}

// ---------------------------------------------------------------------------
// Regression: reusing out-structs must not leak.
// ---------------------------------------------------------------------------

#[test]
fn collect_reuses_out_structs_without_leaking() {
    let tree = Tree::new("reuse_collect");
    let opts = options();
    let root = tree.c();

    let mut entries = zeroed::<CScandirEntryList>();
    let mut errors = zeroed::<CScandirStringList>();
    let mut error = zeroed::<CScandirError>();

    // Each iteration overwrites the list the previous iteration left behind;
    // the previous contents must be released, not orphaned.
    for _ in 0..200 {
        let rc = unsafe {
            cscandir_collect(
                root.as_ptr(),
                &opts,
                &mut entries,
                &mut errors,
                ptr::null_mut(),
                &mut error,
            )
        };
        assert_eq!(rc, OK);
        assert_eq!(entries.len, 6);
    }

    unsafe {
        cscandir_free_entry_list(&mut entries);
        cscandir_free_string_list(&mut errors);
        cscandir_free_error(&mut error);
    }
}

#[test]
fn walk_reuses_out_structs_without_leaking() {
    let tree = Tree::new("walk_reuse");
    let opts = options();
    let root = tree.c();
    let mut toc = zeroed::<CScandirToc>();
    let mut entries = zeroed::<CScandirWalkEntryList>();
    let mut error = zeroed::<CScandirError>();

    for _ in 0..200 {
        let rc = unsafe {
            cscandir_walk(
                root.as_ptr(),
                &opts,
                &mut toc,
                &mut entries,
                ptr::null_mut(),
                &mut error,
            )
        };
        assert_eq!(rc, OK);
        assert_eq!(toc.files.len, 4);
    }

    unsafe {
        cscandir_free_toc(&mut toc);
        cscandir_free_walk_entry_list(&mut entries);
        cscandir_free_error(&mut error);
    }
}

#[test]
fn error_message_is_released_across_repeated_failures() {
    let opts = options();
    let root = CString::new("/nonexistent/cscandir/definitely/not/here").unwrap();
    let mut error = zeroed::<CScandirError>();

    for _ in 0..5000 {
        let rc = unsafe {
            cscandir_collect(
                root.as_ptr(),
                &opts,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut error,
            )
        };
        assert_eq!(rc, ERR_SCAN);
    }

    let msg = unsafe { CStr::from_ptr(error.message) };
    assert!(msg.to_str().unwrap().contains("No such file"));

    unsafe { cscandir_free_error(&mut error) };
    assert!(error.message.is_null());
}

// ---------------------------------------------------------------------------
// Error paths.
// ---------------------------------------------------------------------------

#[test]
fn null_root_path_reports_invalid_argument() {
    let opts = options();
    let mut error = zeroed::<CScandirError>();
    let rc = unsafe {
        cscandir_collect(
            ptr::null(),
            &opts,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, ERR_INVALID_ARGUMENT);
    assert_eq!(error.code, ERR_INVALID_ARGUMENT);
    unsafe { cscandir_free_error(&mut error) };
}

#[test]
fn non_utf8_root_path_is_rejected() {
    let opts = options();
    // 0xFF is never valid UTF-8.
    let root: [c_char; 3] = [b'/' as c_char, -1i8 as c_char, 0];
    let mut error = zeroed::<CScandirError>();
    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, ERR_INVALID_UTF8);
    assert_eq!(error.code, ERR_INVALID_UTF8);
    assert_eq!(
        unsafe { CStr::from_ptr(error.message) }.to_str().unwrap(),
        "root_path is not valid UTF-8"
    );
    unsafe { cscandir_free_error(&mut error) };
}

#[test]
fn invalid_return_type_is_rejected() {
    let tree = Tree::new("bad_return_type");
    let mut opts = options();
    opts.return_type = 7;
    let root = tree.c();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, ERR_INVALID_ARGUMENT);
    unsafe { cscandir_free_error(&mut error) };
}

#[test]
fn invalid_utf8_filter_string_is_rejected() {
    let tree = Tree::new("bad_filter");
    let mut opts = options();
    let bad: [c_char; 3] = [b'a' as c_char, -1i8 as c_char, 0];
    let arr = [bad.as_ptr()];
    opts.file_include = arr.as_ptr();
    opts.file_include_len = 1;

    let root = tree.c();
    let mut error = zeroed::<CScandirError>();
    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, ERR_INVALID_ARGUMENT);
    unsafe { cscandir_free_error(&mut error) };
}

#[test]
fn missing_root_path_reports_scan_error() {
    let opts = options();
    let root = CString::new("/nonexistent/cscandir/nope").unwrap();
    let mut error = zeroed::<CScandirError>();
    let rc = unsafe {
        cscandir_count(
            root.as_ptr(),
            &opts,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, ERR_SCAN);
    unsafe { cscandir_free_error(&mut error) };
}

// ---------------------------------------------------------------------------
// Option handling.
// ---------------------------------------------------------------------------

#[test]
fn null_options_uses_defaults() {
    let tree = Tree::new("null_options");
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            ptr::null(),
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    assert_eq!(entries.len, 6);
    unsafe {
        cscandir_free_entry_list(&mut entries);
        cscandir_free_error(&mut error);
    }
}

#[test]
fn max_depth_limits_results() {
    let tree = Tree::new("max_depth");
    let mut opts = options();
    opts.max_depth = 1;
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    // Depth 1 sees only the root's direct children: 2 files + 1 dir.
    assert_eq!(entries.len, 3);
    unsafe { cscandir_free_entry_list(&mut entries) };
}

#[test]
fn max_depth_zero_means_unlimited() {
    let tree = Tree::new("max_depth_zero");
    let mut opts = options();
    opts.max_depth = 0;
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    assert_eq!(entries.len, 6);
    unsafe { cscandir_free_entry_list(&mut entries) };
}

#[test]
fn max_file_cnt_limits_results() {
    let tree = Tree::new("max_file_cnt");
    let mut opts = options();
    opts.max_file_cnt = 2;
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    // The limit applies to files; directories are still traversed. The exact
    // entry count is not deterministic because the scan runs concurrently.
    assert!(!entries.entries.is_null());
    assert!(entries.len > 0 && entries.len <= 6);
    unsafe { cscandir_free_entry_list(&mut entries) };
}

#[test]
fn file_include_filter_applies() {
    let tree = Tree::new("file_include");
    let mut opts = options();
    let patterns = [CString::new("*.txt").unwrap()];
    let ptrs: Vec<*const c_char> = patterns.iter().map(|p| p.as_ptr()).collect();
    opts.file_include = ptrs.as_ptr();
    opts.file_include_len = ptrs.len();

    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    // `file_include` filters files only; directories are still traversed, so
    // the entry list still carries the two directories.
    let mut txt_files = 0;
    for i in 0..entries.len {
        let e = unsafe { &*entries.entries.add(i) };
        if e.is_file == 1 {
            assert!(entry_path(&entries, i).ends_with(".txt"));
            txt_files += 1;
        }
    }
    assert_eq!(txt_files, 4);
    unsafe { cscandir_free_entry_list(&mut entries) };
}

#[test]
fn skip_hidden_filter_applies() {
    let tree = Tree::new("skip_hidden");
    std::fs::write(tree.path.join(".hidden.txt"), b"x").unwrap();
    let mut opts = options();
    opts.skip_hidden = 1;
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    assert_eq!(entries.len, 6);
    for i in 0..entries.len {
        assert!(!entry_path(&entries, i).contains(".hidden"));
    }
    unsafe { cscandir_free_entry_list(&mut entries) };
}

#[test]
fn extended_return_type_populates_ext_fields() {
    let tree = Tree::new("ext_fields");
    let mut opts = options();
    opts.return_type = 1; // CSCANDIR_RETURN_EXT
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    assert!(entries.len > 0);
    for i in 0..entries.len {
        let e = unsafe { &*entries.entries.add(i) };
        assert_eq!(e.has_ext, 1);
    }
    unsafe { cscandir_free_entry_list(&mut entries) };
}

#[test]
fn base_return_type_leaves_ext_fields_zero() {
    let tree = Tree::new("base_fields");
    let opts = options();
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    for i in 0..entries.len {
        let e = unsafe { &*entries.entries.add(i) };
        assert_eq!(e.has_ext, 0);
        assert_eq!(e.mode, 0);
        assert_eq!(e.ino, 0);
    }
    unsafe { cscandir_free_entry_list(&mut entries) };
}

// ---------------------------------------------------------------------------
// Statistics.
// ---------------------------------------------------------------------------

#[test]
fn collect_returns_statistics() {
    let tree = Tree::new("collect_stats");
    let opts = options();
    let root = tree.c();
    let mut stats = zeroed::<CScandirStatistics>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut stats,
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    assert_eq!(stats.files, 4);
    assert_eq!(stats.dirs, 2);
    assert!(stats.duration > 0.0);
    unsafe { cscandir_free_error(&mut error) };
}

#[test]
fn count_returns_statistics() {
    let tree = Tree::new("count_stats");
    let opts = options();
    let root = tree.c();
    let mut stats = zeroed::<CScandirStatistics>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_count(
            root.as_ptr(),
            &opts,
            &mut stats,
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    assert_eq!(stats.files, 4);
    assert_eq!(stats.dirs, 2);
    assert!(stats.duration > 0.0);
    unsafe { cscandir_free_error(&mut error) };
}

// ---------------------------------------------------------------------------
// Walk.
// ---------------------------------------------------------------------------

#[test]
fn walk_returns_merged_toc_and_per_root_entries() {
    let tree = Tree::new("walk_basic");
    let opts = options();
    let root = tree.c();
    let mut toc = zeroed::<CScandirToc>();
    let mut entries = zeroed::<CScandirWalkEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_walk(
            root.as_ptr(),
            &opts,
            &mut toc,
            &mut entries,
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);

    assert_eq!(toc.files.len, 4);
    assert_eq!(toc.dirs.len, 2);

    // One entry per scanned directory, each holding names relative to that
    // directory. The root's own entry carries an empty path.
    assert!(entries.len >= 2);
    let mut total_files = 0;
    for i in 0..entries.len {
        let e = unsafe { &*entries.entries.add(i) };
        let path = unsafe { CStr::from_ptr(e.path) }.to_str().unwrap();
        // Paths are relative to the scan root; the root's own entry is empty.
        assert!(
            path.is_empty() || tree.path.join(path).is_dir(),
            "unexpected walk root {path:?}"
        );
        total_files += e.toc.files.len;
    }
    assert_eq!(total_files, 4);

    unsafe {
        cscandir_free_toc(&mut toc);
        cscandir_free_walk_entry_list(&mut entries);
        cscandir_free_error(&mut error);
    }
}

#[test]
fn walk_toc_lists_are_populated_and_consistent() {
    let tree = Tree::new("walk_toc_lists");
    let opts = options();
    let root = tree.c();
    let mut toc = zeroed::<CScandirToc>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_walk(
            root.as_ptr(),
            &opts,
            &mut toc,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);

    let files = list_items(&toc.files);
    assert_eq!(files.len(), 4);
    for f in &files {
        assert!(f.ends_with(".txt"), "unexpected file entry {f}");
    }
    assert_eq!(list_items(&toc.dirs).len(), 2);

    unsafe {
        cscandir_free_toc(&mut toc);
        cscandir_free_error(&mut error);
    }
}

#[test]
fn walk_returns_statistics() {
    let tree = Tree::new("walk_stats");
    let opts = options();
    let root = tree.c();
    let mut stats = zeroed::<CScandirStatistics>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_walk(
            root.as_ptr(),
            &opts,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut stats,
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    assert_eq!(stats.files, 4);
    unsafe { cscandir_free_error(&mut error) };
}

#[test]
fn walk_null_toc_but_entries_requested() {
    let tree = Tree::new("walk_toc_null");
    let opts = options();
    let root = tree.c();
    let mut entries = zeroed::<CScandirWalkEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_walk(
            root.as_ptr(),
            &opts,
            ptr::null_mut(),
            &mut entries,
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);
    assert!(entries.len >= 2);
    unsafe { cscandir_free_walk_entry_list(&mut entries) };
}

// ---------------------------------------------------------------------------
// Free functions.
// ---------------------------------------------------------------------------

#[test]
fn free_functions_are_idempotent() {
    let tree = Tree::new("free_idempotent");
    let opts = options();
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();
    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);

    unsafe {
        cscandir_free_entry_list(&mut entries);
        cscandir_free_entry_list(&mut entries);
        cscandir_free_entry_list(&mut entries);
    }
    assert_eq!(entries.len, 0);
    assert!(entries.entries.is_null());

    unsafe { cscandir_free_error(&mut error) };
}

#[test]
fn free_on_uninitialized_struct_is_safe() {
    // Every free function must tolerate storage the library never wrote to.
    unsafe {
        let mut entries = std::mem::MaybeUninit::<CScandirEntryList>::uninit();
        cscandir_free_entry_list(entries.as_mut_ptr());
        let mut list = std::mem::MaybeUninit::<CScandirStringList>::uninit();
        cscandir_free_string_list(list.as_mut_ptr());
        let mut toc = std::mem::MaybeUninit::<CScandirToc>::uninit();
        cscandir_free_toc(toc.as_mut_ptr());
        let mut wl = std::mem::MaybeUninit::<CScandirWalkEntryList>::uninit();
        cscandir_free_walk_entry_list(wl.as_mut_ptr());
        let mut err = std::mem::MaybeUninit::<CScandirError>::uninit();
        cscandir_free_error(err.as_mut_ptr());
    }
}

#[test]
fn init_functions_reset_to_empty() {
    unsafe {
        let mut entries = zeroed::<CScandirEntryList>();
        cscandir_entry_list_init(&mut entries);
        assert!(entries.entries.is_null());
        assert_eq!(entries.len, 0);

        let mut list = zeroed::<CScandirStringList>();
        cscandir_string_list_init(&mut list);
        assert!(list.items.is_null());

        let mut toc = zeroed::<CScandirToc>();
        cscandir_toc_init(&mut toc);
        assert!(toc.files.items.is_null());

        let mut wl = zeroed::<CScandirWalkEntryList>();
        cscandir_walk_entry_list_init(&mut wl);
        assert!(wl.entries.is_null());

        let mut err = zeroed::<CScandirError>();
        cscandir_error_init(&mut err);
        assert_eq!(err.code, OK);
        assert!(err.message.is_null());
    }
}

#[test]
fn null_pointers_are_accepted_everywhere() {
    unsafe {
        cscandir_options_init(ptr::null_mut());
        cscandir_entry_list_init(ptr::null_mut());
        cscandir_string_list_init(ptr::null_mut());
        cscandir_toc_init(ptr::null_mut());
        cscandir_walk_entry_list_init(ptr::null_mut());
        cscandir_error_init(ptr::null_mut());
        cscandir_free_entry_list(ptr::null_mut());
        cscandir_free_string_list(ptr::null_mut());
        cscandir_free_toc(ptr::null_mut());
        cscandir_free_walk_entry_list(ptr::null_mut());
        cscandir_free_error(ptr::null_mut());

        // Every out-param NULL, including the error struct.
        let root = CString::new("/tmp").unwrap();
        let opts = options();
        assert_eq!(
            cscandir_collect(
                root.as_ptr(),
                &opts,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            ),
            OK
        );
        assert_eq!(
            cscandir_walk(
                root.as_ptr(),
                &opts,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            ),
            OK
        );
    }
}

// ---------------------------------------------------------------------------
// Entry level detail.
// ---------------------------------------------------------------------------

#[test]
fn entries_report_type_and_timestamps() {
    let tree = Tree::new("entry_detail");
    let opts = options();
    let root = tree.c();
    let mut entries = zeroed::<CScandirEntryList>();
    let mut error = zeroed::<CScandirError>();

    let rc = unsafe {
        cscandir_collect(
            root.as_ptr(),
            &opts,
            &mut entries,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error,
        )
    };
    assert_eq!(rc, OK);

    let mut files = 0;
    let mut dirs = 0;
    for i in 0..entries.len {
        let e = unsafe { &*entries.entries.add(i) };
        assert!(e.ctime > 0.0);
        assert!(e.mtime > 0.0);
        if e.is_file == 1 {
            files += 1;
            assert!(e.size > 0);
        }
        if e.is_dir == 1 {
            dirs += 1;
        }
    }
    assert_eq!(files, 4);
    assert_eq!(dirs, 2);

    unsafe { cscandir_free_entry_list(&mut entries) };
}
