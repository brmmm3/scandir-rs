#![cfg_attr(windows, feature(junction_point))]

use std::io::Error;

use scandir::Count;

mod common;

#[test]
fn test_count() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let count = Count::new(temp_dir.path())?.collect()?;
    assert!(count.errors.is_empty());
    //assert!(count.duration > 0.0); --> Fails on MAC
    assert_eq!(0, count.devices);
    #[cfg(windows)]
    {
        assert_eq!(85, count.files);
        assert_eq!(13, count.dirs);
        assert_eq!(27, count.slinks);
    }
    #[cfg(unix)]
    {
        assert_eq!(81, count.files);
        assert_eq!(12, count.dirs);
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
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let count = Count::new(temp_dir.path())?.skip_hidden(true).collect()?;
    assert!(count.errors.is_empty());
    //assert!(count.duration > 0.0); --> Fails on MAC
    assert_eq!(0, count.devices);
    #[cfg(windows)]
    {
        assert_eq!(67, count.files);
        assert_eq!(13, count.dirs);
        assert_eq!(27, count.slinks);
    }
    #[cfg(unix)]
    {
        assert_eq!(63, count.files);
        assert_eq!(12, count.dirs);
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
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let count = Count::new(temp_dir.path())?.extended(true).collect()?;
    assert!(count.errors.is_empty());
    //assert!(count.duration > 0.0); --> Fails on MAC
    assert_eq!(0, count.devices);
    #[cfg(windows)]
    {
        assert_eq!(40, count.files);
        assert_eq!(13, count.dirs);
        assert_eq!(27, count.slinks);
    }
    #[cfg(unix)]
    {
        assert_eq!(36, count.files);
        assert_eq!(12, count.dirs);
        assert_eq!(54, count.slinks);
        assert_eq!(63, count.pipes);
    }
    assert_eq!(45, count.hlinks);
    common::cleanup(temp_dir)
}

#[test]
fn test_count_follow_links() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 3)?;
    let count = Count::new(temp_dir.path())?.follow_links(true).collect()?;
    assert!(count.errors.is_empty());
    //assert!(count.duration > 0.0); --> Fails on MAC
    assert_eq!(0, count.devices);
    #[cfg(windows)]
    {
        assert_eq!(85, count.files);
        assert_eq!(13, count.dirs);
        assert_eq!(27, count.slinks);
    }
    #[cfg(unix)]
    {
        assert_eq!(81, count.files);
        assert_eq!(12, count.dirs);
        assert_eq!(54, count.slinks);
        assert_eq!(0, count.pipes);
    }
    assert_eq!(0, count.hlinks);
    common::cleanup(temp_dir)
}
