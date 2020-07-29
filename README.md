# `scandir-rs`

``scandir_rs`` is a directory iteration module like ``os.walk()``, but with more features and higher speed. Depending on the function call
it yields a list of paths, tuple of lists grouped by their entry type or ``DirEntry`` objects that include file type and stat information along
with the name. Using ``scandir_rs`` is about **2-17 times faster** than ``os.walk()`` (depending on the platform, file system and file tree structure)
by parallelizing the iteration in background.

If you are just interested in directory statistics you can use the submodule ``count``.

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

Build wheel (not on Windows):

```sh
maturin build --release --strip
```

Build wheel on Windows:

```sh
maturin build --release --strip --no-sdist
```

``maturin`` will build the wheels for all Python versions installed on your system.

## Building and running tests for different Python versions

To make it easier to build wheels for several different Python versions the script ``build_wheels.sh`` has been added.
It creates wheels for Python versions 3.6, 3.7, 3.8 and 3.9. In addition it runs ``pytest`` after successfull creation of each wheel.

To be able to run the script ``pyenv`` needs to be installed first including all Python interpreter versions mentioned above.

Instruction how to install ``pyenv`` can be found [here](https://github.com/pyenv/pyenv).

## Examples

Get statistics of a directory:

```python
import scandir_rs as scandir

print(scandir.count.count("~/workspace", extended=True))
```

The same, but asynchronously in background using a class instance:

```python
import scandir_rs as scandir

scanner = scandir.count.Count("~/workspace", extended=True))
scanner.start())  # Start background thread pool
...
value = scanner.statistics  # Can be read at any time
...
scanner.stop()  # If you want to cancel the scanner
```

and with a context manager:

```python
import scandir_rs as scandir

C = scandir.count.Count("~/workspace", extended=True))
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

with extended data:

```python
import scandir_rs as scandir

for root, dirs, files, symlinks, other, errors in scandir.walk.Walk("~/workspace",
        return_type=scandir.RETURN_TYPE_EXT):
    # Do something
```

``os.scandir()`` example:

```python
import scandir_rs as scandir

for path, entry in scandir.scandir.Scandir("~/workspace",
        return_type=scandir.RETURN_TYPE_EXT):
    # entry is a custom DirEntry object
```

## Benchmarks

See [examples/benchmark.py](https://github.com/brmmm3/scandir-rs/blob/master/examples/benchmark.py)

In the below table the line **scandir_rs.walk.Walk** returns comparable
results to os.walk.

### Linux with Ryzen 5 2400G and SSD

#### Directory */usr* with

- 83790 directories
- 671847 files
- 48480 symlinks
- 1278 hardlinks
- 0 devices
- 0 pipes
- 30.3GB size and 31.9GB usage on disk

| Time [s] | Method                                              |
| -------- | --------------------------------------------------- |
| 5.319    | os.walk (Python 3.8)                                |
| 13.351   | os.walk+os.stat (Python 3.8)                        |
| 0.918    | scandir_rs.count.count                              |
| 1.340    | scandir_rs.count.count(extended=True)               |
| 0.812    | scandir_rs.count.Count                              |
| 1.663    | scandir_rs.walk.toc                                 |
| 1.107    | scandir_rs.walk.Walk (iter)                         |
| 1.775    | scandir_rs.walk.Walk (collect)                      |
| 2.511    | scandir_rs.scandir.entries (RETURN_TYPE_FAST)       |
| 2.561    | scandir_rs.scandir.entries (RETURN_TYPE_BASE)       |
| 2.496    | scandir_rs.scandir.entries (RETURN_TYPE_EXT)        |
| 2.881    | scandir_rs.scandir.entries (RETURN_TYPE_FULL)       |
| 2.437    | scandir_rs.scandir.entries (iter, RETURN_TYPE_FULL) |

#### Directory *linux-5.5.5* with

- 4391 directories
- 66459 files
- 35 symlinks
- 13 hardlinks
- 0 devices
- 0 pipes
- 870.7MB size and 1021.5MB usage on disk

| Time [s] | Method                                              |
| -------- | --------------------------------------------------- |
| 0.343    | os.walk (Python 3.8)                                |
| 0.966    | os.walk+os.stat (Python 3.8)                        |
| 0.067    | scandir_rs.count.count                              |
| 0.116    | scandir_rs.count.count(extended=True)               |
| 0.067    | scandir_rs.count.Count                              |
| 0.155    | scandir_rs.walk.toc                                 |
| 0.081    | scandir_rs.walk.Walk (iter)                         |
| 0.150    | scandir_rs.walk.Walk (collect)                      |
| 0.186    | scandir_rs.scandir.entries (RETURN_TYPE_FAST)       |
| 0.201    | scandir_rs.scandir.entries (RETURN_TYPE_BASE)       |
| 0.202    | scandir_rs.scandir.entries (RETURN_TYPE_EXT)        |
| 0.260    | scandir_rs.scandir.entries (RETURN_TYPE_FULL)       |
| 0.210    | scandir_rs.scandir.entries (iter, RETURN_TYPE_FULL) |

Up to **~5 times faster** on Linux.

### Windows 10 with Laptop Core i7-4810MQ @ 2.8GHz Laptop, MTF SSD

#### Directory *C:\Windows* with

- 130429 directories
- 426588 files
- 0 symlinks
- 53563 hardlinks
- 0 devices
- 0 pipes
- 49.8GB size and 50.9GB usage on disk

| Time [s] | Method                                              |
| -------- | --------------------------------------------------- |
| 96.544   | os.walk (Python 3.8)                                |
| 328.965  | os.walk+os.stat (Python 3.8)                        |
| 17.133   | scandir_rs.count.count                              |
| 90.272   | scandir_rs.count.count(extended=True)               |
| 19.607   | scandir_rs.count.Count                              |
| 19.654   | scandir_rs.walk.toc                                 |
| 18.203   | scandir_rs.walk.Walk (iter)                         |
| 19.704   | scandir_rs.walk.Walk (collect)                      |
| 80.027   | scandir_rs.scandir.entries (RETURN_TYPE_FAST)       |
| 82.822   | scandir_rs.scandir.entries (RETURN_TYPE_BASE)       |
| 84.734   | scandir_rs.scandir.entries (RETURN_TYPE_EXT)        |
| 87.079   | scandir_rs.scandir.entries (RETURN_TYPE_FULL)       |
| 92.622   | scandir_rs.scandir.entries (iter, RETURN_TYPE_FULL) |

#### Directory *linux-5.5.5* with

- 4390 directories
- 66446 files
- 35 symlinks
- 0 hardlinks
- 0 devices
- 0 pipes
- 870.7MB size and 1021.5MB usage on disk

| Time [s] | Method                                              |
| -------- | --------------------------------------------------- |
| 2.679    | os.walk (Python 3.8)                                |
| 17.334   | os.walk+os.stat (Python 3.8)                        |
| 0.383    | scandir_rs.count.count                              |
| 2.890    | scandir_rs.count.count(extended=True)               |
| 0.431    | scandir_rs.count.Count                              |
| 0.574    | scandir_rs.walk.toc                                 |
| 0.441    | scandir_rs.walk.Walk (iter)                         |
| 0.582    | scandir_rs.walk.Walk (collect)                      |
| 3.169    | scandir_rs.scandir.entries (RETURN_TYPE_FAST)       |
| 3.205    | scandir_rs.scandir.entries (RETURN_TYPE_BASE)       |
| 3.169    | scandir_rs.scandir.entries (RETURN_TYPE_EXT)        |
| 3.301    | scandir_rs.scandir.entries (RETURN_TYPE_FULL)       |
| 3.149    | scandir_rs.scandir.entries (iter, RETURN_TYPE_FULL) |

Up to **6 times faster** on Windows 10.
