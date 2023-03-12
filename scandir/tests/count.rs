use std::fs::{create_dir, hard_link, File};
use std::io::Error;
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(unix)]
use unix_named_pipe;

use scandir::Count;

mod common;

#[test]
fn test_count() -> Result<(), Error> {
    let temp_dir = common::setup();
    let dir = temp_dir.path();
    for i in 1..=10 {
        create_dir(dir.join(format!("dir{i}")))?
    }
    for i in 1..=12 {
        drop(File::create(dir.join(format!("file{i}")))?);
    }
    for i in 1..=11 {
        hard_link(
            dir.join(format!("file{i}")),
            dir.join(format!("hardlink{i}")),
        )?;
    }
    #[cfg(unix)]
    for i in 1..=9 {
        symlink(
            dir.join(format!("file{i}")),
            dir.join(format!("symlink{i}")),
        )?;
    }
    #[cfg(unix)]
    for i in 1..=8 {
        unix_named_pipe::create(dir.join(format!("pipe{i}")), None)?;
    }
    let count = Count::new(dir)?.collect()?;
    assert!(count.errors.is_empty());
    assert!(count.duration > 0.0);
    assert_eq!(10, count.dirs);
    assert_eq!(23, count.files);
    #[cfg(unix)]
    assert_eq!(9, count.slinks);
    assert_eq!(0, count.hlinks);
    assert_eq!(0, count.pipes);
    common::cleanup(temp_dir)
}

#[test]
fn test_count_extended() -> Result<(), Error> {
    let temp_dir = common::setup();
    let dir = temp_dir.path();
    for i in 1..=10 {
        create_dir(dir.join(format!("dir{i}")))?
    }
    for i in 1..=12 {
        drop(File::create(dir.join(format!("file{i}")))?);
    }
    for i in 1..=11 {
        hard_link(
            dir.join(format!("file{i}")),
            dir.join(format!("hardlink{i}")),
        )?;
    }
    #[cfg(unix)]
    for i in 1..=9 {
        symlink(
            dir.join(format!("file{i}")),
            dir.join(format!("symlink{i}")),
        )?;
    }
    #[cfg(unix)]
    for i in 1..=8 {
        unix_named_pipe::create(dir.join(format!("pipe{i}")), None)?;
    }
    let count = Count::new(dir)?.extended(true).collect()?;
    assert!(count.errors.is_empty());
    assert!(count.duration > 0.0);
    assert_eq!(10, count.dirs);
    assert_eq!(12, count.files);
    #[cfg(unix)]
    assert_eq!(9, count.slinks);
    assert_eq!(11, count.hlinks);
    assert_eq!(8, count.pipes);
    common::cleanup(temp_dir)
}
