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

IMPORTANT: In order to build this project at least Rust version 1.61 is needed!

**Build wheel:**

Change to directory `pyscandir`.

Build wheel (on Linux):

```sh
maturin build --release --strip
```

Build wheel on Windows:

```sh
maturin build --release --strip --no-sdist
```

``maturin`` will build the wheels for all Python versions installed on your system.

Alternatively you can use the build script `build_wheels.py`. The precondition to run this script is to have `pyenv` installed.
The script can build the wheel for specific Python versions or for all Python versions installed by `pyenv`.
In addition it runs ``pytest`` after successfull creation of each wheel.

```sh
python build_wheels.py
```

By default the script will build the wheel for the current Python interpreter.
If you want to build the wheel for specific Python version(s) by providing the argument `--versions`.

```sh
python build_wheels.py --versions 3.11.8,3.12.2
```

To build the wheel for all installed Python versions:

```sh
python build_wheels.py --versions *
```

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

instance = Count("/usr", return_type=ReturnType.Ext)
instance.start()  # Start scanning the directory in background
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

See [examples/benches/benchmark.py](https://github.com/brmmm3/scandir-rs/blob/master/pyscandir/examples/benches/benchmark.py)

In the below table the line **Walk.iter** returns comparable
results to os.walk.

### Linux with Ryzen 5 2400G (4 phys cores, 8 total cores) and Samsung SSD 960 EVO 250GB (NVME, ext4)

#### Directory */usr* with

- 45060 directories
- 388518 files
- 34937 symlinks
- 177 hardlinks
- 0 devices
- 0 pipes
- 23.16GB size and 24.02GB usage on disk

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.931535 | Count.collect                        |
|   1.48159  | Count(ReturnType=Ext).collect        |
|   4.09372  | os.walk (Python 3.12.2)              |
|  11.3418   | os.walk (stat) (Python 3.12.2)       |
|   0.925864 | Walk.iter                            |
|   0.96183  | Walk(ReturnType=Ext).iter            |
|   1.47056  | Walk.collect                         |
|   1.36103  | Walk(ReturnType=Ext).collect         |
|   8.75475  | scantree (os.scandir, Python 3.12.2) |
|   1.37387  | Scandir.iter                         |
|   1.87683  | Scandir(ReturnType=Ext).iter         |
|   2.16722  | Scandir.collect                      |
|   2.92552  | Scandir(ReturnType=Ext).collect      |

Walk.iter **~4.4 times faster** than os.walk.  
Walk(Ext).iter **~11.8 times faster** than os.walk(stat).  
Scandir.iter **~6.4 times faster** than scantree(os.scandir).

#### Directory *linux-5.9* with

- 4711 directories
- 69973 files
- 38 symlinks
- 1.08GB size and 1.23GB usage on disk

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.153199 | Count.collect                        |
|   0.249917 | Count(ReturnType=Ext).collect        |
|   0.448813 | os.walk (Python 3.12.2)              |
|   1.64711  | os.walk (stat) (Python 3.12.2)       |
|   0.149128 | Walk.iter                            |
|   0.143961 | Walk(ReturnType=Ext).iter            |
|   0.213981 | Walk.collect                         |
|   0.211384 | Walk(ReturnType=Ext).collect         |
|   1.4078   | scantree (os.scandir, Python 3.12.2) |
|   0.251858 | Scandir.iter                         |
|   0.339001 | Scandir(ReturnType=Ext).iter         |
|   0.298834 | Scandir.collect                      |
|   0.431882 | Scandir(ReturnType=Ext).collect      |

Walk.iter **~3.0 times faster** than os.walk.  
Walk(Ext).iter **~11.4 times faster** than os.walk(stat).  
Scandir.iter **~5.6 times faster** than scantree(os.scandir).

### Windows 10 with Laptop Core i7-4810MQ @ 2.8GHz Laptop, MTF SSD

#### Directory *C:\Windows* with

- 165926 directories
- 316866 files
- 35364 hardlinks
- 39.68GB size and 40.53GB usage on disk

|   Time [s] | Method                               |
|------------|--------------------------------------|
|    10.1644 | Count.collect                        |
|    38.04   | Count(ReturnType=Ext).collect        |
|    99.0955 | os.walk (Python 3.12.2)              |
|   238.835  | os.walk (stat) (Python 3.12.2)       |
|    10.0431 | Walk.iter                            |
|    10.007  | Walk(ReturnType=Ext).iter            |
|    11.8813 | Walk.collect                         |
|    11.8674 | Walk(ReturnType=Ext).collect         |
|    66.8014 | scantree (os.scandir, Python 3.12.2) |
|    10.1068 | Scandir.iter                         |
|    37.7527 | Scandir(ReturnType=Ext).iter         |
|    11.3297 | Scandir.collect                      |
|    38.5138 | Scandir(ReturnType=Ext).collect      |

Walk.iter **~9.9 times faster** than os.walk.  
Walk(Ext).iter **~23.9 times faster** than os.walk(stat).  
Scandir.iter **~6.6 times faster** than scantree(os.scandir).

#### Directory *linux-5.9* with

- 4712 directories
- 69998 files
- 1.08GB size and 1.23GB usage on disk

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.237721 | Count.collect                        |
|   1.86161  | Count(ReturnType=Ext).collect        |
|   2.29283  | os.walk (Python 3.12.2)              |
|  17.6911   | os.walk (stat) (Python 3.12.2)       |
|   0.247534 | Walk.iter                            |
|   0.250716 | Walk(ReturnType=Ext).iter            |
|   0.386362 | Walk.collect                         |
|   0.39245  | Walk(ReturnType=Ext).collect         |
|   1.96715  | scantree (os.scandir, Python 3.12.2) |
|   0.26433  | Scandir.iter                         |
|   1.86403  | Scandir(ReturnType=Ext).iter         |
|   0.375734 | Scandir.collect                      |
|   2.08924  | Scandir(ReturnType=Ext).collect      |

Walk.iter **~9.3 times faster** than os.walk.  
Walk(Ext).iter **~70.6 times faster** than os.walk(stat).  
Scandir.iter **~7.4 times faster** than scantree(os.scandir).
