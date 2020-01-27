# The API of submodule ``count``

## ``Statistics``

The ``Statistics`` class is the return value of method ``count``, also of property ``statistics`` and class method ``collect`` of class ``Count``.

### ``Statistics`` has following class members

- ``dirs`` contains number of directories.
- ``files`` contains number of files.
- ``slinks`` contains number of symlinks.
- ``hlinks`` contains number of hardlinks.
- ``devices`` contains number of devices (only relevant on Unix systems).
- ``pipes`` contains number of named pipes (only relevant on Unix systems).
- ``size`` contains total size of all files.
- ``usage`` contains total usage on disk.
- ``errors`` list of access errors (list of strings).
- ``duration`` time taken for scanning (in seconds as a float).

## ``count(root_path: str, skip_hidden: bool = False, extended: bool = False, max_depth: int = 0, dir_include: list = None, dir_exclude: list = None, file_include: list = None, file_exclude: list = None, case_sensitive: bool = True)``

Scans directory provided through parameter ``root_path`` and returns a ``Statistics`` object. This function is blocking and releases the GIL.

### Parameters

- ``root_path`` is directory to scan. ``~`` is allowed on Unix systems.
- ``skip_hidden`` if ``True`` then ignore all hidden files and directories.
- ``extended`` if ``True`` calculate statistcs for ``hardlinks``, ``devices``, ``pipes``, ``size`` and ``usage`` too.
- ``max_depth`` is maximum depth of iteration. If ``0`` then depth limit is disabled.
- ``dir_include`` list of patterns for directories to include.
- ``dir_exclude`` list of patterns for directories to exclude.
- ``file_include`` list of patterns for files to include.
- ``file_exclude`` list of patterns for files to exclude.
- ``case_sensitive`` if `True` then do case sensitive pattern matching.

For valid file patterns see module [glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

## ``Count(root_path: str, skip_hidden: bool = False, extended: bool = False, max_depth: int = 0, dir_include: list = None, dir_exclude: list = None, file_include: list = None, file_exclude: list = None, case_sensitive: bool = True)``

Creates a class object for more control when calculating statistics. Useful when statistics should be calculated in background without blocking the application. The class instance initially does nothing. To start the scan either the method ``start`` has to be called or a context has to be created (``with ClassInstance:``). When the context is closed the background thread is stopped.

### Example usage of the contect manager

```python
import scandir_rs as scandir
C = scandir.count.Count("~/workspace", extended=True))
with C:
    while C.busy():
        statistics = C.statistics
        # Do something
```

### ``statistics``

Returns a ``Statistics`` object with the current statistics.

### ``has_results()``

Returns ``True`` after iteration has been finished.

### ``as_dict()``

Returns statistics as a ``dict``. Result will only contains the keys of which the values are non zero.

### ``collect()``

This does the same as the call of the ``count`` method. It returns a ``Statistics`` object and in addition the statistics are available also within the class instance through the ``statistics`` property. This method is blocking and releases the GIL.

### ``start()``

Start iterating through the directory in background.

### ``stop()``

Stop iterating.

### ``busy()``

Return ``True`` when the iteration thread is running.
