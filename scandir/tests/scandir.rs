use std::io::Error;

use scandir::{ ReturnType, Scandir, ScandirResult };

mod common;

#[test]
fn test_scandir() -> Result<(), Error> {
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?.collect()?;
    assert_eq!(192, entries.results.len());
    assert_eq!(0, entries.errors.len());
    match entries.results.get(0).unwrap() {
        ScandirResult::DirEntry(d) => {
            assert_eq!("dir1", &d.path);
            assert_eq!(true, d.is_dir);
            assert_eq!(4096, d.st_size);
        }
        _ => panic!("Wrong type"),
    }
    common::cleanup(temp_dir)
}

#[test]
fn test_scandir_extended() -> Result<(), Error> {
    let temp_dir = common::create_temp_file_tree(3, 3, 4, 5, 6, 7)?;
    let entries = Scandir::new(temp_dir.path(), Some(true))?
        .return_type(ReturnType::Ext)
        .collect()?;
    assert_eq!(192, entries.results.len());
    assert_eq!(0, entries.errors.len());
    match entries.results.get(0).unwrap() {
        ScandirResult::DirEntryExt(d) => {
            assert_eq!("dir1", &d.path);
            assert_eq!(true, d.is_dir);
            assert_eq!(4096, d.st_size);
        }
        _ => panic!("Wrong type"),
    }
    common::cleanup(temp_dir)
}
