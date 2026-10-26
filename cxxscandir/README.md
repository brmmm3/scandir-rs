# cxxscandir

C++ bindings for `scandir-rs`, built with the [`cxx`](https://cxx.rs/) bridge.
The wrapper calls the Rust `Scandir` and `Count` APIs directly and returns
owning C++-usable result structures.

## C++ examples

From the repository root:

```bash
cargo build -p cxxscandir --release
make -C cxxscandir build
make -C cxxscandir run ROOT=/tmp
```

All crate examples are C++ sources in `examples/*.cc`: `scandir`, `count`,
`walk`, `cifs_ctime_check`, and `benchmark`. The Cargo crate produces a static
library under `target/release/`. The bridge generates its C++ headers under
Cargo's build output directory; the Makefile locates them and links every
example. Run the additional examples directly, for example:

```bash
cxxscandir/bin/count /tmp --ext
cxxscandir/bin/walk /usr
cxxscandir/bin/cifs_ctime_check /mnt/smb
cxxscandir/bin/benchmark /usr 20
```

The benchmark's `collect-drop` case measures the wrapper returning and then
immediately discarding its entry vector; unlike the C binding's no-output mode,
the C++ wrapper still materializes that vector. Override `CARGO`, `CXX`,
`TARGET_DIR`, `PROFILE`, or `CXXFLAGS` as needed.

## API

Include the generated header `cxxscandir/src/lib.rs.h` and call
`cxxscandir::scan(options)` or `cxxscandir::count(options)`. A failed call
(invalid/unavailable root, invalid glob, or other fatal setup/scan error)
throws `std::exception`; per-entry traversal issues are available separately
in `ScanResult::errors` and `Statistics::errors`.

`ScanOptions` contains:

- `root`: root path; defaults to the empty string, which is not a valid root.
- `sorted`, `skip_hidden`, `case_sensitive`, `follow_links`, `extended`.
- `max_depth` and `max_file_count`; zero means unlimited.
- `dir_include`, `dir_exclude`, `file_include`, and `file_exclude` glob lists;
  empty lists disable the corresponding filter.

`scan()` returns entries with root-relative descendant paths and the root as its
first entry. `duration` is the native measured scan duration. Extended entry
metadata is filled only when `extended` is enabled. `count()` returns native
aggregate values; set `extended` when hardlink, device, pipe, disk usage, or
other extended accounting is needed. Count excludes the root directory.

## Validation

```bash
cargo clippy -p cxxscandir --all-targets -- -D warnings
make -C cxxscandir build
make -C cxxscandir run ROOT=/tmp
make -C cxxscandir test
```

The API regression tests are C++ sources under `tests/*.cc`; run them with
`make -C cxxscandir test`.
