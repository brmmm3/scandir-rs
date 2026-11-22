use std::fs;
use std::io;
use std::time::Duration;

use scandir::{Count, Scandir, Walk};

#[test]
fn missing_root_preserves_error_kind() -> io::Result<()> {
    let root = tempfile::tempdir()?;
    let missing = root.path().join("missing");
    assert_eq!(
        Count::new(&missing).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(
        Walk::new(&missing, None).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(
        Scandir::new(&missing, None).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    Ok(())
}

#[test]
fn limits_apply_in_all_collection_modes() -> io::Result<()> {
    let root = tempfile::tempdir()?;
    for dir in ["a", "b", "c"] {
        fs::create_dir(root.path().join(dir))?;
        for i in 0..8 {
            fs::write(root.path().join(dir).join(format!("file{i}")), b"x")?;
        }
    }

    for limit in [1, 5, 24, usize::MAX, 0] {
        let expected = if limit == 0 { 24 } else { limit.min(24) };
        assert_eq!(
            Count::new(root.path())?
                .max_file_cnt(limit)
                .collect()?
                .files as usize,
            expected
        );
        assert_eq!(
            Scandir::new(root.path(), None)?
                .max_file_cnt(limit)
                .collect()?
                .files()
                .count(),
            expected
        );
        assert_eq!(
            Walk::new(root.path(), None)?
                .max_file_cnt(limit)
                .collect()?
                .files
                .len(),
            expected
        );
        assert_eq!(
            Scandir::new(root.path(), None)?
                .max_file_cnt(limit)
                .collect_timeout(Duration::from_secs(30))?
                .expect("scan timed out")
                .files()
                .count(),
            expected
        );
        assert_eq!(
            Walk::new(root.path(), None)?
                .max_file_cnt(limit)
                .collect_timeout(Duration::from_secs(30))?
                .expect("walk timed out")
                .files
                .len(),
            expected
        );
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn links_consume_the_limit_even_with_hardlink_deduplication() -> io::Result<()> {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir()?;
    fs::write(root.path().join("file"), b"x")?;
    fs::hard_link(root.path().join("file"), root.path().join("hardlink"))?;
    symlink("file", root.path().join("symlink"))?;
    let stats = Count::new(root.path())?
        .extended(true)
        .max_file_cnt(2)
        .collect()?;
    assert_eq!(stats.files + stats.hlinks + stats.slinks, 2);
    let walk = Walk::new(root.path(), None)?.max_file_cnt(2).collect()?;
    assert_eq!(walk.files.len() + walk.symlinks.len() + walk.other.len(), 2);
    let walk = Walk::new(root.path(), None)?
        .max_file_cnt(2)
        .collect_timeout(Duration::from_secs(30))?
        .expect("walk timed out");
    assert_eq!(walk.files.len() + walk.symlinks.len() + walk.other.len(), 2);
    let scan = Scandir::new(root.path(), None)?.max_file_cnt(2).collect()?;
    assert_eq!(
        scan.results.iter().filter(|entry| !entry.is_dir()).count(),
        2
    );
    let scan = Scandir::new(root.path(), None)?
        .max_file_cnt(2)
        .collect_timeout(Duration::from_secs(30))?
        .expect("scan timed out");
    assert_eq!(
        scan.results.iter().filter(|entry| !entry.is_dir()).count(),
        2
    );
    Ok(())
}
