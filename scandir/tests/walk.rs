use std::io::Error;

use scandir::Walk;

mod common;

#[test]
fn test_walk() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5)?;
    let toc = Walk::new(temp_dir.path(), Some(true))?.collect()?;
    assert_eq!(12, toc.dirs.len());
    assert_eq!(63, toc.files.len());
    #[cfg(unix)]
    {
        assert_eq!(54, toc.symlinks.len());
        assert_eq!(63, toc.other.len());
    }
    #[cfg(windows)]
    {
        assert_eq!(0, toc.symlinks.len());
        assert_eq!(0, toc.other.len());
    }
    assert_eq!(0, toc.errors.len());
    common::cleanup(temp_dir)
}

#[test]
fn test_walk_not_skip_hidden() -> Result<(), Error> {
    #[cfg(unix)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    #[cfg(windows)]
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5)?;
    let walk = Walk::new(temp_dir.path(), Some(true))?;
    let mut walk = walk.skip_hidden(false);
    let toc = walk.collect()?;
    assert_eq!(12, toc.dirs.len());
    assert_eq!(81, toc.files.len());
    #[cfg(unix)]
    {
        assert_eq!(54, toc.symlinks.len());
        assert_eq!(63, toc.other.len());
    }
    #[cfg(windows)]
    {
        assert_eq!(0, toc.symlinks.len());
        assert_eq!(0, toc.other.len());
    }
    assert_eq!(0, toc.errors.len());
    common::cleanup(temp_dir)
}
