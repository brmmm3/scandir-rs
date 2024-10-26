use std::io::Error;

use scandir::Count;

mod common;

#[test]
fn test_count() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5)?;
    let count = Count::new(temp_dir.path())?.collect()?;
    assert!(count.errors.is_empty());
    //assert!(count.duration > 0.0); --> Fails on MAC
    assert_eq!(12, count.dirs);
    assert_eq!(81, count.files);
    assert_eq!(0, count.devices);
    #[cfg(unix)]
    {
        assert_eq!(54, count.slinks);
        assert_eq!(0, count.pipes);
    }
    assert_eq!(0, count.hlinks);
    common::cleanup(temp_dir)
}

#[test]
fn test_count_skip_hidden() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5)?;
    let count = Count::new(temp_dir.path())?.skip_hidden(true).collect()?;
    assert!(count.errors.is_empty());
    //assert!(count.duration > 0.0); --> Fails on MAC
    assert_eq!(12, count.dirs);
    assert_eq!(63, count.files);
    assert_eq!(0, count.devices);
    #[cfg(unix)]
    {
        assert_eq!(54, count.slinks);
        assert_eq!(0, count.pipes);
    }
    assert_eq!(0, count.hlinks);
    common::cleanup(temp_dir)
}

#[test]
fn test_count_extended() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5)?;
    let count = Count::new(temp_dir.path())?.extended(true).collect()?;
    assert!(count.errors.is_empty());
    //assert!(count.duration > 0.0); --> Fails on MAC
    assert_eq!(12, count.dirs);
    assert_eq!(36, count.files);
    assert_eq!(0, count.devices);
    #[cfg(unix)]
    {
        assert_eq!(54, count.slinks);
        assert_eq!(63, count.pipes);
    }
    assert_eq!(45, count.hlinks);
    common::cleanup(temp_dir)
}
