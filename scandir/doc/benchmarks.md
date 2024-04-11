# Benchmarks

These are the results of the `benchmarks.rs` run.

## Benchmark results on Linux

### Walk linux-5.9

Benchmark results for linux-5.9 file tree for walkdir crate including a metadata call for each entry and
scandir::Walk call.

![](images/linux_walk_linux-5.9.svg)

### Walk usr

Benchmark results for /usr file tree for walkdir crate including a metadata call for each entry and
scandir::Walk call.

![](images/linux_walk_usr.svg)

### Scandir linux-5.9

Benchmark results for linux-5.9 file tree for scan_dir crate including a metadata call for each entry and
scandir::Scandir call with and without collecting extended metadata information.

![](images/linux_scandir_linux-5.9.svg)

### Scandir usr

Benchmark results for /usr file tree for scan_dir crate including a metadata call for each entry and
scandir::Scandir call with and without collecting extended metadata information.

![](images/linux_scandir_usr.svg)

## Benchmark results on Windows 10

### Walk linux-5.9

Benchmark results for linux-5.9 file tree for walkdir crate including a metadata call for each entry and
scandir::Walk call.

![](images/windows_walk_linux-5.9.svg)

### Walk Windows

Benchmark results for /usr file tree for walkdir crate including a metadata call for each entry and scandir::Walk call.

![](images/windows_walk_windows.svg)

### Scandir linux-5.9

Benchmark results for linux-5.9 file tree for scan_dir crate including a metadata call for each entry and
scandir::Scandir call with and without collecting extended metadata information.

![](images/windows_scandir_linux-5.9.svg)

### Scandir Windows

Benchmark results for /usr file tree for scan_dir crate including a metadata call for each entry and
scandir::Scandir call with and without collecting extended metadata information.

![](images/windows_scandir_windows.svg)
