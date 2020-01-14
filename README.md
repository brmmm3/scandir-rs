# `scandir-rs`

``scandir_rs`` is a directory iteration module like ``os.walk()``,
but with more features and higher speed. Depending on the function call
it yields a list of paths, tuple of lists grouped by their entry type or ``DirEntry`` objects that include file type and stat information along
with the name. Using ``scandir_rs`` is about **2-17 times faster** than ``os.walk()`` (depending on the platform, file system and file tree structure) by parallelizing the iteration in background.

If your are just interested in directory statistics you can use the submodule ``count``.

``scandir_rs`` contains following submodules:

- ``count`` for determining statistics of a directory.
- ``walk`` for getting names of directory entries.
- ``scandir`` for getting detailed stats of directory entries.

For the API see:

- Submodule ``count`` [doc/count.md](https://github.com/brmmm3/scandir-rs/blob/master/doc/count.md)
- Submodule ``walk`` [doc/walk.md](https://github.com/brmmm3/scandir-rs/blob/master/doc/walk.md)
- Submodule ``scandir`` [doc/scandir.md](https://github.com/brmmm3/scandir-rs/blob/master/doc/scandir.md)

## Installation

For building this wheel from source you need Rust with channel ``nightly`` and the tool ``maturin``.

Switch to channel ``nightly``:

```sh
rustup default nightly
```

Install ``maturin``:

```sh
cargo install maturin
```

Build wheel:

```sh
maturin build --release
```

``maturin`` will build the wheels for all Python versions installed on your system.

## Examples

Get statistics of a directory:

```python
import scandir_rs as scandir

print(scandir.count.count("~/workspace", metadata_ext=True))
```

The same, but asynchronously in background using a class instance:

```python
import scandir_rs as scandir

scanner = scandir.count.Count("~/workspace", metadata_ext=True))
scanner.start())  # Start background thread pool
...
value = scanner.statistics  # Can be read at any time
...
scanner.stop()  # If you want to cancel the scanner
```

and with a context manager:

```python
import scandir_rs as scandir

C = scandir.count.Count("~/workspace", metadata_ext=True))
with C:
    while C.busy():
        statistics = C.statistics
        # Do something
```

``os.walk()`` example:

```python
import scandir_rs as scandir

for root, dirs, files in scandir.walk.Walk("~/workspace"):
    # Do something
```

``os.scandir()`` example:

```python
import scandir_rs as scandir

for entry in scandir.scandir.Scandir("~/workspace", metadata_ext=True):
    # Do something
```

## Benchmarks

See [examples/benchmark.py](https://github.com/brmmm3/scandir-rs/blob/master/examples/benchmark.py)

In the below table the line **scandir_rs.walk.Walk** returns comparable
results to os.walk.

### Linux with Ryzen 5 2400G and SSD

#### Directory *~/workspace* with

- 22845 directories
- 321354 files
- 130 symlinks
- 22849 hardlinks
- 4 devices
- 1 pipes
- 4.6GB size and 5.4GB usage on disk

| Time [s] | Method                                              |
|----------|-----------------------------------------------------|
| 0.547    | os.walk (Python 3.7)                                |
| 0.132    | scandir_rs.count.count                              |
| 0.142    | scandir_rs.count.Count                              |
| 0.237    | scandir_rs.walk.Walk                                |
| 0.224    | scandir_rs.walk.toc                                 |
| 0.242    | scandir_rs.walk.collect                             |
| 0.262    | scandir_rs.scandir.entries                          |
| 0.344    | scandir_rs.scandir.entries(metadata=True)           |
| 0.336    | scandir_rs.scandir.entries(metadata_ext=True)       |
| 0.280    | scandir_rs.scandir.Scandir.collect                  |
| 0.262    | scandir_rs.scandir.Scandir.iter                     |
| 0.330    | scandir_rs.scandir.Scandir.iter(metadata_ext=True)  |

Up to **2 times faster** on Linux.

### Windows 10 with Laptop Core i7-4810MQ @ 2.8GHz Laptop, MTF SSD

#### Directory *C:\Windows* with

- 84248 directories
- 293108 files
- 44.4GB size and 45.2GB usage on disk

| Time [s] | Method                                              |
|----------|-----------------------------------------------------|
| 26.881   | os.walk (Python 3.7)                                |
| 4.094    | scandir_rs.count.count                              |
| 3.654    | scandir_rs.count.Count                              |
| 3.978    | scandir_rs.walk.Walk                                |
| 3.848    | scandir_rs.walk.toc                                 |
| 3.777    | scandir_rs.walk.collect                             |
| 3.987    | scandir_rs.scandir.entries                          |
| 3.905    | scandir_rs.scandir.entries(metadata=True)           |
| 4.062    | scandir_rs.scandir.entries(metadata_ext=True)       |
| 3.934    | scandir_rs.scandir.Scandir.collect                  |
| 3.981    | scandir_rs.scandir.Scandir.iter                     |
| 3.821    | scandir_rs.scandir.Scandir.iter(metadata_ext=True)  |

Up to **6.7 times faster** on Windows 10.

#### Directory *C:\testdir* with

- 185563 directories
- 1641277 files
- 2696 symlinks
- 97GB size and 100.5GB usage on disk

| Time [s] | Method                                              |
|----------|-----------------------------------------------------|
| 151.143  | os.walk (Python 3.7)                                |
| 7.549    | scandir_rs.count.count                              |
| 7.531    | scandir_rs.count.Count                              |
| 8.710    | scandir_rs.walk.Walk                                |
| 8.625    | scandir_rs.walk.toc                                 |
| 8.599    | scandir_rs.walk.collect                             |
| 9.014    | scandir_rs.scandir.entries                          |
| 9.208    | scandir_rs.scandir.entries(metadata=True)           |
| 8.925    | scandir_rs.scandir.entries(metadata_ext=True)       |
| 9.243    | scandir_rs.scandir.Scandir.collect                  |
| 8.462    | scandir_rs.scandir.Scandir.iter                     |
| 8.380    | scandir_rs.scandir.Scandir.iter(metadata_ext=True)  |

Up to **17.4 times faster** on Windows 10.
