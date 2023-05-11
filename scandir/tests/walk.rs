use std::io::Error;

use scandir::Walk;

mod common;

#[test]
fn test_walk() -> Result<(), Error> {
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    let toc = Walk::new(temp_dir.path())?.collect(true)?;
    assert_eq!(12, toc.dirs.len());
    assert_eq!(63, toc.files.len());
    assert_eq!(54, toc.symlinks.len());
    assert_eq!(63, toc.other.len());
    assert_eq!(0, toc.errors.len());
    common::cleanup(temp_dir)
}

#[test]
fn test_walk_not_skip_hidden() -> Result<(), Error> {
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    let toc = Walk::new(temp_dir.path())?
        .skip_hidden(false)
        .collect(true)?;
    assert_eq!(12, toc.dirs.len());
    assert_eq!(81, toc.files.len());
    assert_eq!(54, toc.symlinks.len());
    assert_eq!(63, toc.other.len());
    assert_eq!(0, toc.errors.len());
    common::cleanup(temp_dir)
}
