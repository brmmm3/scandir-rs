use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use scandir::{Count, Scandir, Walk};

#[test]
fn unicode_paths_are_relative_and_filters_work() -> io::Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("røøt");
    let nested = PathBuf::from("données");
    std::fs::create_dir_all(root.join(&nested))?;
    std::fs::write(root.join(&nested).join("élève.txt"), b"x")?;
    let expected = nested.join("élève.txt");

    assert_eq!(Count::new(&root)?.collect()?.files, 1);
    for timed in [false, true] {
        let mut walk = Walk::new(&root, None)?.dir_include(Some(vec!["données/**".to_owned()]));
        let toc = if timed {
            walk.collect_timeout(Duration::from_secs(30))?
                .expect("walk timed out")
        } else {
            walk.collect()?
        };
        assert!(toc.errors.is_empty(), "{:?}", toc.errors);
        assert_eq!(toc.files.as_slice(), std::slice::from_ref(&expected));
        let mut scan = Scandir::new(&root, None)?.dir_include(Some(vec!["données/**".to_owned()]));
        let entries = if timed {
            scan.collect_timeout(Duration::from_secs(30))?
                .expect("scan timed out")
        } else {
            (*scan.collect()?).clone()
        };
        assert!(entries.errors.is_empty(), "{:?}", entries.errors);
        assert_eq!(entries.files().next().unwrap().path(), expected);
    }
    Ok(())
}

#[cfg(all(
    windows,
    any(feature = "speedy", feature = "bincode", feature = "json")
))]
#[test]
fn unpaired_windows_surrogates_are_native_and_serialization_fails() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let path = PathBuf::from(OsString::from_wide(&[0x0061, 0xd800]));
    let entry = scandir::DirEntry {
        path: path.clone(),
        ..Default::default()
    };
    assert_eq!(entry.path, path);
    #[cfg(feature = "speedy")]
    assert!(entry.to_speedy().is_err());
    #[cfg(feature = "bincode")]
    assert!(entry.to_vec().is_err());
    #[cfg(feature = "json")]
    assert!(entry.to_json().is_err());
}

#[cfg(windows)]
#[test]
fn unpaired_windows_surrogates_in_filesystem_paths_are_preserved() -> io::Result<()> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let temp = tempfile::tempdir()?;
    let root_name = OsString::from_wide(&[0x72, 0x6f, 0x6f, 0x74, 0xd800]);
    let name = OsString::from_wide(&[0x66, 0x69, 0x6c, 0x65, 0xd801]);
    let root = temp.path().join(root_name);
    std::fs::create_dir(&root)?;
    std::fs::write(root.join(&name), b"x")?;
    assert_eq!(Count::new(&root)?.collect()?.files, 1);
    let walk = Walk::new(&root, None)?.collect()?;
    assert_eq!(walk.files, [PathBuf::from(&name)]);
    let scan = Scandir::new(&root, None)?.collect()?;
    assert_eq!(scan.files().next().unwrap().path(), Path::new(&name));
    Ok(())
}

#[cfg(unix)]
#[test]
fn non_unicode_roots_and_entries_are_lossless_in_all_modes() -> io::Result<()> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = tempfile::tempdir()?;
    let root = temp.path().join(OsString::from_vec(b"root-\xff".to_vec()));
    let nested = PathBuf::from(OsString::from_vec(b"dir-\xfe".to_vec()));
    let first = OsString::from_vec(b"file-\xff".to_vec());
    let second = OsString::from_vec(b"file-\xfe".to_vec());
    std::fs::create_dir_all(root.join(&nested))?;
    std::fs::write(root.join(&first), b"1")?;
    std::fs::write(root.join(&second), b"2")?;
    std::fs::write(root.join(&nested).join(&first), b"3")?;
    let expected = [
        PathBuf::from(&first),
        PathBuf::from(&second),
        nested.join(&first),
    ];

    assert_eq!(Count::new(&root)?.collect()?.files, 3);
    for timed in [false, true] {
        let mut walk = Walk::new(&root, None)?;
        let toc = if timed {
            walk.collect_timeout(Duration::from_secs(30))?
                .expect("walk timed out")
        } else {
            walk.collect()?
        };
        assert!(toc.errors.is_empty(), "{:?}", toc.errors);
        assert_eq!(toc.files.len(), expected.len());
        for path in &expected {
            assert!(toc.files.contains(path), "missing {path:?}");
        }

        let mut scan = Scandir::new(&root, None)?;
        let results = if timed {
            scan.collect_timeout(Duration::from_secs(30))?
                .expect("scan timed out")
        } else {
            (*scan.collect()?).clone()
        };
        assert!(results.errors.is_empty(), "{:?}", results.errors);
        let paths: Vec<&Path> = results.files().map(|entry| entry.path()).collect();
        assert_eq!(paths.len(), expected.len());
        for path in &expected {
            assert!(paths.contains(&path.as_path()), "missing {path:?}");
        }
    }

    let single = Scandir::new(root.join(&first), None)?.collect()?;
    assert_eq!(single.results[0].path(), Path::new(&first));

    let mut walk = Walk::new(&root, None)?;
    walk.start()?;
    let mut batches = Vec::new();
    while walk.busy() {
        batches.extend(walk.results_timeout(Duration::from_millis(100)));
    }
    batches.extend(walk.results(true));
    assert!(batches.iter().any(|(directory, toc)| {
        directory == &nested && toc.files.contains(&PathBuf::from(&first))
    }));
    Ok(())
}

#[cfg(unix)]
#[test]
fn filters_use_relative_paths_when_root_is_non_unicode() -> io::Result<()> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = tempfile::tempdir()?;
    let root = temp.path().join(OsString::from_vec(b"root-\xff".to_vec()));
    std::fs::create_dir_all(root.join("good"))?;
    std::fs::write(root.join("good/hit.txt"), b"x")?;
    let filter = Some(vec!["good/**".to_owned()]);
    assert_eq!(
        Count::new(&root)?
            .dir_include(filter.clone())
            .collect()?
            .files,
        1
    );
    let walk = Walk::new(&root, None)?
        .dir_include(filter.clone())
        .collect()?;
    assert!(walk.errors.is_empty());
    assert_eq!(walk.files, [PathBuf::from("good/hit.txt")]);
    let scan = Scandir::new(&root, None)?.dir_include(filter).collect()?;
    assert!(scan.errors.is_empty());
    assert_eq!(
        scan.files().next().unwrap().path(),
        Path::new("good/hit.txt")
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn unicode_globs_report_non_unicode_names_without_silently_skipping_them() -> io::Result<()> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let root = tempfile::tempdir()?;
    let invalid = OsString::from_vec(b"bad-\xff".to_vec());
    std::fs::create_dir(root.path().join(&invalid))?;
    std::fs::write(root.path().join(&invalid).join("nested.txt"), b"x")?;
    std::fs::write(
        root.path().join(OsString::from_vec(b"file-\xff".to_vec())),
        b"x",
    )?;
    std::fs::write(root.path().join("good.txt"), b"x")?;

    let file_filter = Some(vec!["*.txt".to_owned()]);
    let count = Count::new(root.path())?
        .file_include(file_filter.clone())
        .collect()?;
    assert_eq!(count.files, 2);
    assert!(
        count
            .errors
            .iter()
            .any(|e| e.contains("Unicode file filter"))
    );
    let scan = Scandir::new(root.path(), None)?
        .file_include(file_filter.clone())
        .collect()?;
    assert!(
        scan.errors
            .iter()
            .any(|(_, e)| e.contains("Unicode file filter"))
    );
    let walk = Walk::new(root.path(), None)?
        .file_include(file_filter)
        .collect()?;
    assert!(
        walk.errors
            .iter()
            .any(|e| e.contains("Unicode file filter"))
    );

    let dir_filter = Some(vec!["**".to_owned()]);
    let count = Count::new(root.path())?
        .dir_include(dir_filter.clone())
        .collect()?;
    assert_eq!(count.files, 2);
    assert!(
        count
            .errors
            .iter()
            .any(|e| e.contains("Unicode directory filter"))
    );
    let scan = Scandir::new(root.path(), None)?
        .dir_include(dir_filter.clone())
        .collect()?;
    assert!(
        scan.errors
            .iter()
            .any(|(_, e)| e.contains("Unicode directory filter"))
    );
    let walk = Walk::new(root.path(), None)?
        .dir_include(dir_filter)
        .collect()?;
    assert!(
        walk.errors
            .iter()
            .any(|e| e.contains("Unicode directory filter"))
    );
    Ok(())
}

#[cfg(feature = "speedy")]
#[test]
fn speedy_keeps_string_wire_format_for_unicode_paths() {
    use speedy::{Readable, Writable};
    use std::time::SystemTime;

    #[derive(Readable, Writable)]
    struct PreviousEntry {
        path: String,
        is_symlink: bool,
        is_dir: bool,
        is_file: bool,
        st_ctime: Option<SystemTime>,
        st_mtime: Option<SystemTime>,
        st_atime: Option<SystemTime>,
        st_size: u64,
    }
    #[derive(Readable, Writable)]
    struct PreviousToc {
        dirs: Vec<String>,
        files: Vec<String>,
        symlinks: Vec<String>,
        other: Vec<String>,
        errors: Vec<String>,
    }
    #[derive(Readable, Writable)]
    enum PreviousResult {
        DirEntry(PreviousEntry),
        DirEntryExt(PreviousEntry),
        Error((String, String)),
    }
    #[derive(Readable, Writable)]
    struct PreviousResults {
        results: Vec<PreviousResult>,
        errors: Vec<(String, String)>,
    }
    let entry = scandir::DirEntry {
        path: PathBuf::from("a/file"),
        is_file: true,
        st_size: 42,
        ..Default::default()
    };
    let previous = PreviousEntry {
        path: "a/file".to_owned(),
        is_symlink: false,
        is_dir: false,
        is_file: true,
        st_ctime: None,
        st_mtime: None,
        st_atime: None,
        st_size: 42,
    };
    let bytes = entry.to_speedy().unwrap();
    assert_eq!(bytes, previous.write_to_vec().unwrap());
    assert_eq!(scandir::DirEntry::read_from_buffer(&bytes).unwrap(), entry);

    let mut toc = scandir::Toc::new();
    toc.files.push("a/file".into());
    let previous_toc = PreviousToc {
        dirs: vec![],
        files: vec!["a/file".into()],
        symlinks: vec![],
        other: vec![],
        errors: vec![],
    };
    let results = scandir::ScandirResults {
        results: vec![scandir::ScandirResult::DirEntry(entry)],
        errors: vec![("a".into(), "failed".into())],
    };
    let previous_results = PreviousResults {
        results: vec![PreviousResult::DirEntry(previous)],
        errors: vec![("a".into(), "failed".into())],
    };
    assert_eq!(
        toc.to_speedy().unwrap(),
        previous_toc.write_to_vec().unwrap()
    );
    assert_eq!(
        results.to_speedy().unwrap(),
        previous_results.write_to_vec().unwrap()
    );
    assert_eq!(
        scandir::Toc::read_from_buffer(&toc.to_speedy().unwrap()).unwrap(),
        toc
    );
    assert_eq!(
        scandir::ScandirResults::read_from_buffer(&results.to_speedy().unwrap()).unwrap(),
        results
    );
}

#[cfg(all(unix, any(feature = "speedy", feature = "bincode", feature = "json")))]
#[test]
fn string_serializers_reject_non_unicode_paths() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let path = PathBuf::from(OsString::from_vec(b"bad-\xff".to_vec()));
    let entry = scandir::DirEntry {
        path: path.clone(),
        ..Default::default()
    };
    let mut toc = scandir::Toc::new();
    toc.files.push(path.clone());
    let errors = scandir::ScandirResults {
        results: vec![],
        errors: vec![(path, "failed".to_owned())],
    };
    #[cfg(feature = "speedy")]
    {
        assert!(entry.to_speedy().is_err());
        assert!(toc.to_speedy().is_err());
        assert!(errors.to_speedy().is_err());
    }
    #[cfg(feature = "bincode")]
    {
        assert!(entry.to_vec().is_err());
        assert!(toc.to_bincode().is_err());
        assert!(errors.to_bincode().is_err());
    }
    #[cfg(feature = "json")]
    {
        assert!(entry.to_json().is_err());
        assert!(toc.to_json().is_err());
        assert!(errors.to_json().is_err());
    }
}
