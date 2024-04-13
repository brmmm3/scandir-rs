# `scandir-rs`

`scandir-rs` is a Rust project which provides a [Rust](scandir/README.md) and a [Python](pyscandir/README.md) module for directory iteration, like `os.walk()` or `os.scandir()`, but with more features and higher speed. Depending on the function call it yields a list of paths, tuple of lists grouped by their entry type or `DirEntry` objects that include file type and stat information along with the file name. Directory iteration is **many** times faster than `os.walk()`, `os.scandir()`, `walkdir` or `scan_dir` (see benchmarks for [Rust](scandir/doc/benchmarks.md) and [Python](pyscandir/doc/benchmarks.md)).

The higher performance is achieved by parallelizing the file system access for reducing the access delays.
