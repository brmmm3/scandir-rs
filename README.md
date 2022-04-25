# `scandir-rs`

``scandir_rs`` is a directory iteration module like ``os.walk()``, but with more features and higher speed. Depending on the function call
it yields a list of paths, tuple of lists grouped by their entry type or ``DirEntry`` objects that include file type and stat information along
with the name. Using ``scandir_rs`` is about **2-17 times faster** than ``os.walk()`` (depending on the platform, file system and file tree structure)
by parallelizing the iteration in background.

If you are just interested in directory statistics you can use the ``Count``.

``scandir_rs`` contains following classes:

- ``Count`` for determining statistics of a directory.
- ``Walk`` for getting names of directory entries.
- ``Scandir`` for getting detailed stats of directory entries.

For the API see:

- Class ``Count`` [doc/count.md](https://github.com/brmmm3/scandir-rs/blob/master/doc/count.md)
- Class ``Walk`` [doc/walk.md](https://github.com/brmmm3/scandir-rs/blob/master/doc/walk.md)
- Class ``Scandir`` [doc/scandir.md](https://github.com/brmmm3/scandir-rs/blob/master/doc/scandir.md)

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
from scandir_rs import Count, ReturnType

print(Count("/usr", return_type=ReturnType.Ext).collect())
```

The same, but asynchronously in background using a class instance:

```python
from scandir_rs import Count, ReturnType

instance = Count("/usr", return_type=ReturnType.Ext))
instance.start())  # Start background thread pool
...
values = instance.results()  # Can be read at any time
...
instance.stop()  # If you want to cancel the scanner
```

and with a context manager:

```python
from scandir_rs import Count, ReturnType

instance = Count("/usr", return_type=ReturnType.Ext))
with instance:
    while instance.busy():
        statistics = instance.results()
        # Do something
```

``os.walk()`` example:

```python
from scandir_rs import Walk

for root, dirs, files in Walk("/usr"):
    # Do something
```

with extended data:

```python
from scandir_rs import Walk, ReturnType

for root, dirs, files, symlinks, other, errors in Walk("/usr", return_type=ReturnType.Ext):
    # Do something
```

``os.scandir()`` example:

```python
from scandir_rs import Scandir, ReturnType

for path, entry in Scandir("~/workspace", return_type=ReturnType.Ext):
    # entry is a custom DirEntry object
```

## Benchmarks

See [examples/benchmark.py](https://github.com/brmmm3/scandir-rs/blob/master/examples/benchmark.py)

In the below table the line **Walk.iter** returns comparable
results to os.walk.

### Linux with Ryzen 5 2400G and SSD

#### Directory */usr* with

- 105521 directories
- 841030 files
- 47753 symlinks
- 1215 hardlinks
- 12 devices
- 0 pipes
- 41.3GB size and 43.3GB usage on disk

| Time [s] | Method                          |
| -------- | ------------------------------- |
| 2.487    | os.walk (Python 3.10)           |
| 0.425    | Count.collect                   |
| 0.777    | Count(ReturnType=Ext).collect   |
| 0.655    | Walk.iter                       |
| 0.879    | Walk.collect                    |
| 0.812    | Walk(ReturnType=Ext).collect    |
| 1.528    | Scandir.collect                 |
| 1.591    | Scandir.iter                    |
| 1.751    | Scandir(ReturnType=Ext).collect |

Around **3.8 times faster** on Linux (os.walk compared to Walk.iter).

### Windows 10 with Laptop Core i7-4810MQ @ 2.8GHz Laptop, MTF SSD

#### Directory *C:\Windows* with

- 84248 directories
- 293108 files
- 44.4GB size and 45.2GB usage on disk

| Time [s] | Method                                             |
| -------- | -------------------------------------------------- |
| 26.881   | os.walk (Python 3.7)                               |
| 4.094    | scandir_rs.count.count                             |
| 3.654    | scandir_rs.count.Count                             |
| 3.978    | scandir_rs.walk.Walk                               |
| 3.848    | scandir_rs.walk.toc                                |
| 3.777    | scandir_rs.walk.collect                            |
| 3.987    | scandir_rs.scandir.entries                         |
| 3.905    | scandir_rs.scandir.entries(metadata=True)          |
| 4.062    | scandir_rs.scandir.entries(metadata_ext=True)      |
| 3.934    | scandir_rs.scandir.Scandir.collect                 |
| 3.981    | scandir_rs.scandir.Scandir.iter                    |
| 3.821    | scandir_rs.scandir.Scandir.iter(metadata_ext=True) |

Up to **6.7 times faster** on Windows 10.

#### Directory *C:\testdir* with

- 185563 directories
- 1641277 files
- 2696 symlinks
- 97GB size and 100.5GB usage on disk

| Time [s] | Method                                             |
| -------- | -------------------------------------------------- |
| 151.143  | os.walk (Python 3.7)                               |
| 7.549    | scandir_rs.count.count                             |
| 7.531    | scandir_rs.count.Count                             |
| 8.710    | scandir_rs.walk.Walk                               |
| 8.625    | scandir_rs.walk.toc                                |
| 8.599    | scandir_rs.walk.collect                            |
| 9.014    | scandir_rs.scandir.entries                         |
| 9.208    | scandir_rs.scandir.entries(metadata=True)          |
| 8.925    | scandir_rs.scandir.entries(metadata_ext=True)      |
| 9.243    | scandir_rs.scandir.Scandir.collect                 |
| 8.462    | scandir_rs.scandir.Scandir.iter                    |
| 8.380    | scandir_rs.scandir.Scandir.iter(metadata_ext=True) |

Up to **17.4 times faster** on Windows 10.
