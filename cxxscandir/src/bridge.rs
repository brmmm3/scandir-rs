// Bridge definition for cxx - defines the C++ interface
// This file is used by cxx-build to generate the C++ header and Rust bindings.

#[cxx::bridge]
mod ffi {
    // Error codes from the C library
    enum ErrorCode: i32 {
        Ok = 0,
        InvalidArgument = 1,
        InvalidUtf8 = 2,
        Scan = 3,
        NulByte = 4,
    }

    enum ReturnType: u32 {
        Base = 0,
        Ext = 1,
    }

    // Statistics structure
    struct Statistics {
        dirs: i32,
        files: i32,
        slinks: i32,
        hlinks: i32,
        devices: i32,
        pipes: i32,
        size: u64,
        usage: u64,
        duration: f64,
    }

    // Entry structure
    struct Entry {
        path: String,
        is_symlink: bool,
        is_dir: bool,
        is_file: bool,
        ctime: f64,
        mtime: f64,
        atime: f64,
        size: u64,
        has_ext: bool,
        mode: u32,
        ino: u64,
        dev: u64,
        nlink: u64,
        blksize: u64,
        blocks: u64,
        uid: u32,
        gid: u32,
        rdev: u64,
    }

    // TOC structure
    struct Toc {
        dirs: Vec<String>,
        files: Vec<String>,
        symlinks: Vec<String>,
        other: Vec<String>,
        errors: Vec<String>,
    }

    // WalkEntry structure
    struct WalkEntry {
        path: String,
        toc: Toc,
    }

    // Options structure
    struct Options {
        sorted: bool,
        skip_hidden: bool,
        max_depth: usize,
        max_file_cnt: usize,
        dir_include: Vec<String>,
        dir_exclude: Vec<String>,
        file_include: Vec<String>,
        file_exclude: Vec<String>,
        case_sensitive: bool,
        follow_links: bool,
        return_type: ReturnType,
    }

    // Error type
    struct Error {
        code: i32,
        message: String,
    }

    // CollectResult
    struct CollectResult {
        entries: Vec<Entry>,
        errors: Vec<String>,
        statistics: Statistics,
    }

    // WalkResult
    struct WalkResult {
        toc: Toc,
        roots: Vec<WalkEntry>,
        statistics: Statistics,
    }

    extern "Rust" {
        // Options functions
        fn options_default() -> Options;

        // Main entry points
        fn collect(root: &str, options: &Options) -> Result<CollectResult, Error>;
        fn count(root: &str, options: &Options) -> Result<Statistics, Error>;
        fn walk(root: &str, options: &Options) -> Result<WalkResult, Error>;
    }
}