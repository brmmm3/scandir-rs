# `scandir-rs`

`scandir-rs` is a Rust project which provides a [Rust](https://github.com/brmmm3/scandir-rs/blob/master/scandir/README.md)
 and a [Python](https://github.com/brmmm3/scandir-rs/blob/master/pyscandir/README.md) module for
 directory iteration, like `os.walk()` or `os.scandir()`, but with more features and higher speed.
 Depending on the function call it yields a list of paths, tuple of lists grouped by their entry
 type or `DirEntry` objects that include file type and stat information along with the file name.
 Directory iteration is **many** times faster than `os.walk()`, `os.scandir()`, `walkdir` or
 `scan_dir` (see **benchmarks** for [Rust](https://github.com/brmmm3/scandir-rs/blob/master/scandir/doc/benchmarks.md)
 and [Python](https://github.com/brmmm3/scandir-rs/blob/master/pyscandir/doc/benchmarks.md)).

The higher performance is achieved through parallelizing the file system access for reducing the
 access delay because of the overhead each file access has.

**Note:** `scandir_rs` uses libc 2.34, which is currently not supported by the manylinux releases.
 So it is not possible to upload prebuilt Linux wheels to PyPI. As a workaround you can download
 the Linux wheels from [github](https://github.com/brmmm3/scandir-rs/releases/tag/2.7.0).

**Note:** Since 2.8.0 `skip_hidden` is now `false` by default!

## Python examples

### Count

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

### Walk

```python
from scandir_rs import Walk

for root, dirs, files in Walk("/usr"):
    # Do something
```

#### with extended metadata

```python
from scandir_rs import Walk, ReturnType

for root, dirs, files, symlinks, other, errors in Walk("/usr", return_type=ReturnType.Ext):
    # Do something
```

### Scandir

```python
from scandir_rs import Scandir, ReturnType

for path, entry in Scandir("~/workspace", return_type=ReturnType.Ext):
    # entry is a custom DirEntry object
```

or collecting all the result:

```python
from scandir_rs import Scandir, ReturnType

instance = Scandir("~/workspace")
instance.extended(True)
resuolts = instance.collect()
```

## Rust examples

### Count

```rust
let mut instance = Count::new(&root_dir)?;
// Exclude directories dir0 and dir1
instance = instance.dir_exclude(Some(vec!["dir0".to_owned(), "dir1".to_owned()]));
// Use extended metadata for calculating statistics
instance = instance.extended(true);
// Start and wait for finishing background worker thread.
// collect checks if background thread is already running. If not it will be started.
let statistics = instance.collect()?;
```

### Walk

```rust
let mut instance = Walk::new(&root_dir, None)?;
// Use extended metadata for calculating statistics
instance = instance.extended(true);
// Start background thread for traversing file tree
instance.start()?;
loop {
    if !instance.busy() {
        break;
    }
    // Do something...
    thread::sleep(Duration::from_millis(10));
}
let result = instance.collect()?;
```

### Scandir

```rust
let mut instance = Scandir::new(&root_dir, None)?;
// Use extended metadata for calculating statistics
instance = instance.extended(true);
// Start background thread for traversing file tree
instance.start()?;
loop {
    if !instance.busy() {
        break;
    }
    // Do something...
    thread::sleep(Duration::from_millis(10));
}
let result = instance.collect()?;
```
