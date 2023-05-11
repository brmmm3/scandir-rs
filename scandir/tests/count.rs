use std::io::Error;

use scandir::Count;

mod common;

#[test]
fn test_count() -> Result<(), Error> {
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    let count = Count::new(temp_dir.path())?.collect()?;
    assert!(count.errors.is_empty());
    assert!(count.duration > 0.0);
    assert_eq!(12, count.dirs);
    assert_eq!(63, count.files);
    assert_eq!(0, count.devices);
    #[cfg(unix)]
    assert_eq!(54, count.slinks);
    assert_eq!(0, count.hlinks);
    #[cfg(unix)]
    assert_eq!(0, count.pipes);
    common::cleanup(temp_dir)
}

#[test]
fn test_count_extended() -> Result<(), Error> {
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    let count = Count::new(temp_dir.path())?.extended(true).collect()?;
    assert!(count.errors.is_empty());
    assert!(count.duration > 0.0);
    assert_eq!(12, count.dirs);
    assert_eq!(36, count.files);
    assert_eq!(0, count.devices);
    #[cfg(unix)]
    assert_eq!(54, count.slinks);
    assert_eq!(27, count.hlinks);
    #[cfg(unix)]
    assert_eq!(63, count.pipes);
    common::cleanup(temp_dir)
}
