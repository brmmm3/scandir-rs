//! C++ bindings for the scandir-rs scanner using the `cxx` bridge.

#[cxx::bridge(namespace = "cxxscandir")]
pub mod ffi {
    /// Options shared between C++ and Rust. An empty filter vector means no
    /// filtering. Zero depth/file limits mean unlimited.
    #[derive(Clone, Debug, Default)]
    struct ScanOptions {
        root: String,
        sorted: bool,
        skip_hidden: bool,
        max_depth: u64,
        max_file_count: u64,
        dir_include: Vec<String>,
        dir_exclude: Vec<String>,
        file_include: Vec<String>,
        file_exclude: Vec<String>,
        case_sensitive: bool,
        follow_links: bool,
        extended: bool,
    }

    /// An individual entry. Extended fields are populated when
    /// `ScanOptions::extended` is true.
    #[derive(Clone, Debug, Default)]
    struct Entry {
        path: String,
        is_symlink: bool,
        is_dir: bool,
        is_file: bool,
        ctime: f64,
        mtime: f64,
        atime: f64,
        btime: f64,
        size: u64,
        blksize: u64,
        blocks: u64,
        mode: u32,
        nlink: u64,
        uid: u32,
        gid: u32,
        inode: u64,
        device: u64,
        rdevice: u64,
    }

    /// Entry collection and nonfatal per-entry traversal errors.
    #[derive(Clone, Debug, Default)]
    struct ScanResult {
        entries: Vec<Entry>,
        errors: Vec<String>,
        duration: f64,
    }

    /// Aggregate statistics from the native Count scanner.
    #[derive(Clone, Debug, Default)]
    struct Statistics {
        dirs: i32,
        files: i32,
        symlinks: i32,
        hardlinks: i32,
        devices: i32,
        pipes: i32,
        size: u64,
        usage: u64,
        duration: f64,
        errors: Vec<String>,
    }

    extern "Rust" {
        /// Scan a directory tree and return entries and nonfatal traversal errors.
        fn scan(options: ScanOptions) -> Result<ScanResult>;

        /// Count a directory tree without materializing entry results.
        fn count(options: ScanOptions) -> Result<Statistics>;
    }
}

fn apply_scan_options(
    mut scanner: scandir::Scandir,
    options: &ffi::ScanOptions,
) -> scandir::Scandir {
    scanner = scanner
        .sorted(options.sorted)
        .skip_hidden(options.skip_hidden)
        .max_depth(usize::try_from(options.max_depth).unwrap_or(usize::MAX))
        .max_file_cnt(usize::try_from(options.max_file_count).unwrap_or(usize::MAX))
        .case_sensitive(options.case_sensitive)
        .follow_links(options.follow_links)
        .extended(options.extended);

    if !options.dir_include.is_empty() {
        scanner = scanner.dir_include(Some(options.dir_include.clone()));
    }
    if !options.dir_exclude.is_empty() {
        scanner = scanner.dir_exclude(Some(options.dir_exclude.clone()));
    }
    if !options.file_include.is_empty() {
        scanner = scanner.file_include(Some(options.file_include.clone()));
    }
    if !options.file_exclude.is_empty() {
        scanner = scanner.file_exclude(Some(options.file_exclude.clone()));
    }
    scanner
}

fn apply_count_options(mut counter: scandir::Count, options: &ffi::ScanOptions) -> scandir::Count {
    counter = counter
        .skip_hidden(options.skip_hidden)
        .max_depth(usize::try_from(options.max_depth).unwrap_or(usize::MAX))
        .max_file_cnt(usize::try_from(options.max_file_count).unwrap_or(usize::MAX))
        .case_sensitive(options.case_sensitive)
        .follow_links(options.follow_links)
        .extended(options.extended);

    if !options.dir_include.is_empty() {
        counter = counter.dir_include(Some(options.dir_include.clone()));
    }
    if !options.dir_exclude.is_empty() {
        counter = counter.dir_exclude(Some(options.dir_exclude.clone()));
    }
    if !options.file_include.is_empty() {
        counter = counter.file_include(Some(options.file_include.clone()));
    }
    if !options.file_exclude.is_empty() {
        counter = counter.file_exclude(Some(options.file_exclude.clone()));
    }
    counter
}

fn scan(options: ffi::ScanOptions) -> Result<ffi::ScanResult, String> {
    let mut scanner =
        scandir::Scandir::new(&options.root, Some(true)).map_err(|error| error.to_string())?;
    scanner = apply_scan_options(scanner, &options);
    let results = scanner.collect().map_err(|error| error.to_string())?;
    let duration = scanner.duration();
    drop(scanner);
    let results = std::sync::Arc::try_unwrap(results).unwrap_or_else(|shared| (*shared).clone());

    let entries = results
        .results
        .into_iter()
        .map(|result| match result {
            scandir::ScandirResult::DirEntry(entry) => {
                let ctime = entry.ctime();
                let mtime = entry.mtime();
                let atime = entry.atime();
                ffi::Entry {
                    path: entry.path,
                    is_symlink: entry.is_symlink,
                    is_dir: entry.is_dir,
                    is_file: entry.is_file,
                    ctime,
                    mtime,
                    atime,
                    size: entry.st_size,
                    ..Default::default()
                }
            }
            scandir::ScandirResult::DirEntryExt(entry) => {
                let ctime = entry.ctime();
                let mtime = entry.mtime();
                let atime = entry.atime();
                let btime = entry.btime();
                ffi::Entry {
                    path: entry.path,
                    is_symlink: entry.is_symlink,
                    is_dir: entry.is_dir,
                    is_file: entry.is_file,
                    ctime,
                    mtime,
                    atime,
                    btime,
                    size: entry.st_size,
                    blksize: entry.st_blksize,
                    blocks: entry.st_blocks,
                    mode: entry.st_mode,
                    nlink: entry.st_nlink,
                    uid: entry.st_uid,
                    gid: entry.st_gid,
                    inode: entry.st_ino,
                    device: entry.st_dev,
                    rdevice: entry.st_rdev,
                }
            }
            scandir::ScandirResult::Error(_) => {
                unreachable!("Scandir::collect stores traversal errors separately from results")
            }
        })
        .collect();

    let errors = results
        .errors
        .into_iter()
        .map(|(path, message)| {
            if path.is_empty() {
                message
            } else {
                format!("{path}: {message}")
            }
        })
        .collect();
    Ok(ffi::ScanResult {
        entries,
        errors,
        duration,
    })
}

fn count(options: ffi::ScanOptions) -> Result<ffi::Statistics, String> {
    let mut counter = scandir::Count::new(&options.root).map_err(|error| error.to_string())?;
    counter = apply_count_options(counter, &options);
    let stats = counter.collect().map_err(|error| error.to_string())?;
    Ok(ffi::Statistics {
        dirs: stats.dirs,
        files: stats.files,
        symlinks: stats.slinks,
        hardlinks: stats.hlinks,
        devices: stats.devices,
        pipes: stats.pipes,
        size: stats.size,
        usage: stats.usage,
        duration: stats.duration,
        errors: stats.errors,
    })
}
