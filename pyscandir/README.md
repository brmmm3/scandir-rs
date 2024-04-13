# scandir-rs

The Python module is called `scandir_rs` and installable via `pip`. It is an alternative to `os.walk()` and `os.scandir()` with more features and higher speed. On Linux it is **3 - 11 times faster** and on Windows **6 - 70 time faster** (see [benchmarks](doc/benchmarks.md)).  
It releases the GIL and the scanning is done in a background thread. With different methods intermediate results can be read.

If you are just interested in directory statistics you can use the `Count`.

`scandir_rs` contains following classes:

- `Count` for determining statistics of a directory.
- `Walk` for getting names of directory entries.
- `Scandir` for getting detailed stats of directory entries.

For the API see:

- Class [Count](doc/count.md)
- Class [Walk](doc/walk.md)
- Class [Scandir](doc/scandir.md)

## Installation

For building this wheel from source you need the tool `maturin`.

Install `maturin`:

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
