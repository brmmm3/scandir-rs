use std::io::Error;

use scandir::{ReturnType, Scandir, ScandirResult};

mod common;

#[test]
fn test_scandir() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?.collect()?;
    #[cfg(unix)]
    assert_eq!(210, entries.results.len());
    #[cfg(windows)]
    assert_eq!(93, entries.results.len());
    assert_eq!(0, entries.errors.len());
    #[cfg(target_os = "linux")]
    match entries.results.first().unwrap() {
        ScandirResult::DirEntry(d) => {
            assert_eq!("dir1", &d.path);
            assert!(d.is_dir);
            #[cfg(target_os = "linux")]
            assert_eq!(4096, d.st_size);
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
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5)?;
    let scandir = Scandir::new(temp_dir.path(), Some(true))?;
    let mut scandir = scandir.skip_hidden(true);
    let entries = scandir.collect()?;
    #[cfg(unix)]
    assert_eq!(192, entries.results.len());
    #[cfg(windows)]
    assert_eq!(75, entries.results.len());
    assert_eq!(0, entries.errors.len());
    match entries.results.first().unwrap() {
        ScandirResult::DirEntry(d) => {
            assert!(vec!["dir1", "dir2", "dir3"].contains(&d.path.as_str()));
            assert!(d.is_dir);
            #[cfg(target_os = "linux")]
            assert_eq!(4096, d.st_size);
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
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?
        .return_type(ReturnType::Ext)
        .collect()?;
    #[cfg(unix)]
    assert_eq!(210, entries.results.len());
    #[cfg(windows)]
    assert_eq!(93, entries.results.len());
    assert_eq!(0, entries.errors.len());
    match entries.results.first().unwrap() {
        ScandirResult::DirEntryExt(d) => {
            assert!(vec!["dir1", "dir2", "dir3"].contains(&d.path.as_str()));
            assert!(d.is_dir);
            #[cfg(target_os = "linux")]
            assert_eq!(4096, d.st_size);
            #[cfg(target_os = "macos")]
            assert_eq!(96, d.st_size);
            #[cfg(windows)]
            assert_eq!(0, d.st_size);
        }
        _ => panic!("Wrong type"),
    }
    common::cleanup(temp_dir)
}
