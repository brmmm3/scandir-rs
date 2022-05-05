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

For building this wheel from source you need the tool ``maturin``.

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
It creates wheels for Python versions 3.7, 3.8, 3.9, 3.10 and 3.11.0a7. In addition it runs ``pytest`` after successfull creation of each wheel.

To be able to run the script ``pyenv`` needs to be installed first including the following Python interpreter versions:

3.7.9, 3.8.10, 3.9.12, 3.10.4, 3.11.a07

Instruction how to install ``pyenv`` can be found [here](https://github.com/pyenv/pyenv).

## Examples

Get statistics of a directory:

```python
from scandir_rs import Count, ReturnType

print(Count("/usr", return_type=ReturnType.Ext).collect())
```

The `collect` method releases the GIL. So other Python threads can run in parallel.

The same, but asynchronously in background using a class instance:

```python
from scandir_rs import Count, ReturnType

instance = Count("/usr", return_type=ReturnType.Ext))
instance.start())  # Start scanning the directory
...
values = instance.results()  # Returns the current statistics. Can be read at any time
...
if instance.busy():  # Check if the task is still running.
...
instance.stop()  # If you want to cancel the task
...
instance.join()  # Wait for the instance to finish.
```

and with a context manager:

```python
import time

from scandir_rs import Count, ReturnType

with Count("/usr", return_type=ReturnType.Ext) as instance:
    while instance.busy():
        statistics = instance.results()
        # Do something
        time.sleep(0.01)
    print(instance.results())
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

- 110171 directories
- 862634 files
- 47804 symlinks
- 12275 hardlinks
- 12 devices
- 0 pipes
- 32.7GB size and 34.8GB usage on disk

| Time [s] | Method                          |
| -------- | ------------------------------- |
| 3.450    | os.walk (Python 3.10)           |
| 6.021    | scantree (Python 3.10)          |
| 1.186    | Count.collect                   |
| 1.416    | Count(ReturnType=Ext).collect   |
| 1.089    | Walk.iter                       |
| 1.350    | Walk.collect                    |
| 1.336    | Walk(ReturnType=Ext).collect    |
| 2.232    | Scandir.collect                 |
| 1.839    | Scandir.iter                    |
| 2.437    | Scandir(ReturnType=Ext).collect |

Around **~3 times faster** on Linux (os.walk compared to Walk.iter).

### Windows 10 with Laptop Core i7-4810MQ @ 2.8GHz Laptop, MTF SSD

#### Directory *C:\Windows* with

- 132604 directories
- 349911 files
- 44.4GB size and 45.2GB usage on disk

| Time [s] | Method                          |
| -------- | ------------------------------- |
| 21.779   | os.walk (Python 3.10)           |
| 13.085   | scantree (Python 3.10)          |
| 3.257    | Count.collect                   |
| 16.605   | Count(ReturnType=Ext).collect   |
| 4.102    | Walk.iter                       |
| 4.056    | Walk.collect                    |
| 4.190    | Walk(ReturnType=Ext).collect    |
| 3.993    | Scandir.collect                 |
| 8.921    | Scandir.iter                    |
| 17.616   | Scandir(ReturnType=Ext).collect |

Around **~5.3 times faster** on Windows 10 (os.walk compared to Walk.iter).

#### Directory *linux-5.9* with

- 4711 directories
- 69973 files
- 1.08GB size and 1.23GB usage on disk

| Time [s] | Method                                                     |
| -------- | ---------------------------------------------------------- |
| 0.411    | os.walk (Python 3.10)                                      |
| 1.203    | os.walk (stat)                                             |
| 0.218    | scandir.Count()                                            |
| 0.278    | scandir.Count(return_type=ReturnType.Ext).collect()        |
| 0.227    | scandir_rs.Walk().collect()                                |
| 0.164    | scandir.Walk(return_type=scandir.ReturnType.Ext) (iter)    |
| 0.204    | scandir.Walk(return_type=scandir.ReturnType.Ext) (collect) |
| 0.350    | scandir.Scandir(return_type=ReturnType.Base).collect()     |
| 0.426    | scandir.Scandir(return_type=ReturnType.Ext).collect()      |

Around **~2.5 times faster** on Linux (os.walk compared to Walk.iter).


| Time [s] | Method                                                     |
| -------- | ---------------------------------------------------------- |
| 1.998    | os.walk (Python 3.10)                                      |
| 14.875   | os.walk (stat)                                             |
| 0.278    | scandir.Count()                                            |
| 2.114    | scandir.Count(return_type=ReturnType.Ext).collect()        |
| 0.464    | scandir_rs.Walk().collect()                                |
| 0.313    | scandir.Walk(return_type=scandir.ReturnType.Ext) (iter)    |
| 0.455    | scandir.Walk(return_type=scandir.ReturnType.Ext) (collect) |
| 0.624    | scandir.Scandir(return_type=ReturnType.Base).collect()     |
| 2.409    | scandir.Scandir(return_type=ReturnType.Ext).collect()      |

Around **~6.4 times faster** on Windows 10 (os.walk compared to Walk.iter).
