---
title: Python
---

The `scandir_rs` PyO3 extension supports Python 3.10 and later. `Count`
provides statistics, `Walk` yields directory-grouped names, and `Scandir`
iterates file entries with metadata. Scanning runs outside the GIL, and the
worker APIs can expose intermediate results.

## Install

```sh
python -m pip install scandir-rs
```

If a compatible wheel is not available for your platform, install Rust and
Maturin, activate a virtual environment, then build the extension from a
checkout:

```sh
python -m pip install maturin
cd pyscandir
maturin develop --release
```

Linux wheel availability depends on the platform's libc and wheel compatibility.
Consult the [Python package guide](https://github.com/brmmm3/scandir-rs/blob/master/pyscandir/README.md)
for current wheel/build details.

## Count

```python
from scandir_rs import Count, ReturnType

stats = Count("/var/log", return_type=ReturnType.Ext).collect()
print(stats.files, stats.dirs, stats.size)
```

For asynchronous access, start a worker and read its current statistics while
it runs:

```python
import time

from scandir_rs import Count

counter = Count("/var/log")
counter.start()
while counter.busy:
    current = counter.results()
    # Update a progress display or perform other work.
    time.sleep(0.01)
stats = counter.collect()
```

The unreleased 3.0.0 source adds `collect(timeout=seconds)`, which returns
`None` if that wait times out; the worker continues running. This timeout
argument may not be present in the published package you install. Call `stop()`
to cancel a running worker, or `join()` to wait for it.

## Iterate files or walk directories

```python
from scandir_rs import ReturnType, Scandir, Walk

for entry in Scandir("/var/log", return_type=ReturnType.Ext):
    print(entry.path, entry.is_dir, entry.st_size)

for root, dirs, files in Walk("/var/log"):
    print(root, dirs, files)
```

With `return_type=ReturnType.Ext`, `Walk` yields additional type groups and
errors:

```python
for root, dirs, files, symlinks, other, errors in Walk(
    "/var/log", return_type=ReturnType.Ext
):
    ...
```

Both classes accept options such as `sorted`, `skip_hidden`, `max_depth`,
`max_file_cnt`, directory/file include and exclude globs, `case_sensitive`, and
`follow_links`. For a full collected result use `Scandir(...).collect()`; the
returned `ScandirResults` exposes entries grouped by type and traversal errors.
In the unreleased 3.0.0 source, this returns `ScandirResults`; the published
2.10.1 package returns an `(entries, errors)` tuple instead.

## Defaults and references

Hidden entries are included by default. Use `ReturnType.Base` for basic
metadata; `ReturnType.Ext` requests extended stat fields. The wheel includes
type stubs for editors and static type checkers. See the [Python README](https://github.com/brmmm3/scandir-rs/blob/master/pyscandir/README.md)
and [API reference](https://github.com/brmmm3/scandir-rs/tree/master/pyscandir/doc)
for serialization helpers and all method signatures.
