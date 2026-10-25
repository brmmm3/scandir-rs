#![cfg_attr(windows, feature(junction_point))]

use std::io::Error;

use scandir::{ReturnType, Scandir, ScandirResult};

mod common;

#[test]
fn test_scandir() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?.collect()?;
    #[cfg(unix)]
    assert_eq!(211, entries.results.len());
    #[cfg(windows)]
    assert_eq!(125, entries.results.len());
    assert_eq!(0, entries.errors.len());
    #[cfg(target_os = "linux")]
    match entries.results.first().unwrap() {
        ScandirResult::DirEntry(d) => {
            assert_eq!("dir3", &d.path);
            assert!(d.is_dir);
            #[cfg(target_os = "linux")]
            assert!(d.st_size <= 4096); // Directories on tmpfs can have a size smaller than 4096
            #[cfg(target_os = "macos")]
            assert_eq!(96, d.st_size);
            #[cfg(windows)]
            assert_eq!(0, d.st_size);
        }
        _ => panic!("Wrong type"),
    }
    common::cleanup(temp_dir)
}

#[test]
fn test_scandir_skip_hidden() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let scandir = Scandir::new(temp_dir.path(), Some(true))?;
    let mut scandir = scandir.skip_hidden(true);
    let entries = scandir.collect()?;
    #[cfg(unix)]
    assert_eq!(193, entries.results.len());
    #[cfg(windows)]
    assert_eq!(107, entries.results.len());
    assert_eq!(0, entries.errors.len());
    match entries.results.first().unwrap() {
        ScandirResult::DirEntry(d) => {
            assert!(["dir1", "dir2", "dir3"].contains(&d.path.as_str()));
            assert!(d.is_dir);
            #[cfg(target_os = "linux")]
            assert!(d.st_size <= 4096); // Directories on tmpfs can have a size smaller than 4096
            #[cfg(target_os = "macos")]
            assert_eq!(96, d.st_size);
            #[cfg(windows)]
            assert_eq!(0, d.st_size);
        }
        _ => panic!("Wrong type"),
    }
    common::cleanup(temp_dir)
}

#[test]
fn test_scandir_extended() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?
        .return_type(ReturnType::Ext)
        .collect()?;
    #[cfg(unix)]
    assert_eq!(211, entries.results.len());
    #[cfg(windows)]
    assert_eq!(125, entries.results.len());
    assert_eq!(0, entries.errors.len());
    match entries.results.first().unwrap() {
        ScandirResult::DirEntryExt(d) => {
            assert!(["dir1", "dir2", "dir3"].contains(&d.path.as_str()));
            assert!(d.is_dir);
            #[cfg(target_os = "linux")]
            assert!(d.st_size <= 4096); // Directories on tmpfs can have a size smaller than 4096
            #[cfg(target_os = "macos")]
            assert_eq!(96, d.st_size);
            #[cfg(windows)]
            assert_eq!(0, d.st_size);
        }
        _ => panic!("Wrong type"),
    }
    common::cleanup(temp_dir)
}

#[test]
fn test_scandir_follow_links() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?
        .follow_links(true)
        .collect()?;
    #[cfg(unix)]
    assert_eq!(211, entries.results.len());
    #[cfg(windows)]
    assert_eq!(234, entries.results.len());
    assert_eq!(0, entries.errors.len());
    #[cfg(target_os = "linux")]
    match entries.results.first().unwrap() {
        ScandirResult::DirEntry(d) => {
            assert_eq!("dir3", &d.path);
            assert!(d.is_dir);
            #[cfg(target_os = "linux")]
            assert!(d.st_size <= 4096); // Directories on tmpfs can have a size smaller than 4096
            #[cfg(target_os = "macos")]
            assert_eq!(96, d.st_size);
            #[cfg(windows)]
            assert_eq!(0, d.st_size);
        }
        _ => panic!("Wrong type"),
    }
    common::cleanup(temp_dir)
}

#[test]
fn test_scandir_categorizes_dirs_and_files() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?.collect()?;
    assert!(
        entries.dirs().next().is_some(),
        "dirs should contain subdirectories"
    );
    assert!(
        entries.files().next().is_some(),
        "files should contain regular files"
    );
    assert_eq!(
        entries.results.len(),
        entries.dirs().count()
            + entries.files().count()
            + entries.symlinks().count()
            + entries.other().count(),
        "results total should equal sum of categorized fields"
    );
    assert_eq!(0, entries.errors.len());
    common::cleanup(temp_dir)
}

#[test]
fn test_scandir_categorization_types() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?.collect()?;
    for entry in entries.dirs() {
        assert!(entry.is_dir(), "entry in dirs should be a directory");
    }
    for entry in entries.files() {
        assert!(entry.is_file(), "entry in files should be a file");
    }
    for entry in entries.symlinks() {
        assert!(entry.is_symlink(), "entry in symlinks should be a symlink");
    }
    common::cleanup(temp_dir)
}

#[test]
fn test_scandir_results_backward_compat() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?.collect()?;
    assert!(
        !entries.results.is_empty(),
        "results field should still contain all entries"
    );
    #[cfg(unix)]
    assert_eq!(211, entries.results.len());
    #[cfg(windows)]
    assert_eq!(126, entries.results.len());
    let categorized = entries.dirs().count()
        + entries.files().count()
        + entries.symlinks().count()
        + entries.other().count();
    assert_eq!(
        entries.results.len(),
        categorized,
        "sum of categorized should equal results"
    );
    common::cleanup(temp_dir)
}

#[test]
fn test_scandir_ctime_not_none() -> Result<(), Error> {
    // Regression for Linux CIFS nounix bug: io_uring.rs requested only STATX_BASIC_STATS
    // (omits STATX_BTIME) but old code filled st_ctime from stx_btime -> st_ctime was None.
    // Now st_ctime comes from POSIX change time (stx_ctime, always filled by STATX_BASIC_STATS),
    // and st_btime comes from stx_btime (filled when STATX_BTIME requested; None on fs/kernel
    // without btime support like tmpfs/ext3). Windows: both fields come from creation_time.
    use scandir::ScandirResult;
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(1, 1, 1, 0, 0, 0)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(1, 1, 1, 0, 0)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?
        .return_type(ReturnType::Ext)
        .collect()?;
    assert_eq!(0, entries.errors.len(), "no scan errors expected");
    let file_entry = entries
        .files()
        .find_map(|r| match r {
            ScandirResult::DirEntryExt(d) if d.is_file => Some(d),
            _ => None,
        })
        .expect("should have at least one regular file");
    assert!(
        file_entry.st_ctime.is_some(),
        "st_ctime must not be None (regression: was None on Linux CIFS nounix)"
    );
    assert!(
        file_entry.ctime() > 0.0,
        "ctime must not be 0.0 (regression: was 0.0 on Linux CIFS nounix)"
    );
    // st_btime field exists and is callable (compile-time check).
    // Value semantics: Some on Windows (creation_time always set); may be None on Linux tmpfs
    // (no btime concept) but Some on ext4/CIFS nounix SMB2/3 with kernel>=4.13.
    let _btime_secs: f64 = file_entry.btime();
    #[cfg(windows)]
    assert!(
        file_entry.st_btime.is_some(),
        "st_btime must not be None on Windows (creation_time is always set)"
    );
    common::cleanup(temp_dir)
}
