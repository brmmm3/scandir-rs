# The API of class `Count`

## `Statistics`

The `Statistics` class is the return value of class methods `results` and `collect`
of class `Count`.

### `Statistics` has following class members

- `dirs` contains number of directories.
- `files` contains number of files.
- `slinks` contains number of symlinks.
- `hlinks` contains number of hardlinks.
- `devices` contains number of devices (only relevant on Unix systems).
- `pipes` contains number of named pipes (only relevant on Unix systems).
- `size` contains total size of all files.
- `usage` contains total usage on disk.
- `errors` list of access errors (list of strings).
- `duration` time taken for scanning (in seconds as a float).

## `Count()`

    def Count(
    root_path: str,
    skip_hidden: bool = False,
    max_depth: int = 0,
    max_file_cnt: int = 0,
    dir_include: List[str] | None = None,
    dir_exclude: List[str] | None = None,
    file_include: List[str] | None = None,
    file_exclude: List[str] | None = None,
    case_sensitive: bool = False,
    return_type: ReturnType = ReturnType.Base,
    )

Creates a class instance for calculating statistics. The class instance initially does nothing.
To start the scan either the method `start`  or the method `collect` has to be called or a
context has to be created (`with Count(...) as instance:`). When the context is closed the
background thread is stopped.

### Parameters

- `root_path` is directory to scan. `~` is allowed on Unix systems.
- `skip_hidden` if `True` then ignore all hidden files and directories.
- `max_depth` is maximum depth of iteration. If `0` then depth limit is disabled.
- `max_file_cnt` is maximum number of files to collect. If `0` then limit is disabled.
- `dir_include` list of patterns for directories to include.
- `dir_exclude` list of patterns for directories to exclude.
- `file_include` list of patterns for files to include.
- `file_exclude` list of patterns for files to exclude.
- `case_sensitive` if `True` then do case sensitive pattern matching.
- `follow_links` if `True` then follow symlinks and junctions.
- `return_type` defines type of data returned.

For valid file patterns see module [glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

### Return types

- `ReturnType.Base` calculate statistcs for `dirs`, `files`, `slinks`, `size` and `usage`.
- `ReturnType.Ext` in addition to above calculate statistcs `hlinks` and on Unix platforms
`devices` and `pipes`.

### Example usage of the context manager

``python
import scandir_rs as scandir

with scandir.Count("~/workspace", extended=True) as instance:
    while instance.busy():
        statistics = instance.results()

## Do something

``

## `clear()`

Clear all results.

### `start()`

Start calculating statistics in background. Raises an expception if a task is already running.

### `join()`

Wait for parsing task to finish.

### `stop()`

Stop parsing task.

### `collect() -> Statistics`

Calculate statistics and return a `Statistics` object when the task has finished.
This method is blocking and releases the GIL.

### `has_results() -> bool`

Returns `True` if new statistics are available.

### `results() -> Statistics`

Return a `Statistics` object with the current statistics.

### `has_errors() -> bool`

Returns `True` if errors occured while scanning the directory tree. The errors can be found
in the statistics object.

### `duration -> float`

Returns the duration of the task in seconds as float. As long as the task is running it will
return 0.

### `finished -> bool`

Returns `True` after the task has finished.

### `busy -> bool`

Returns `True` while a task is running.

### `as_dict() -> dict`

Returns statistics as a `dict`. Result will only contain the keys of which the values are non zero.

### `to_speedy() -> bytes`

Feature `speedy` enabled.

Returns statistics as [speedy](https://docs.rs/speedy/latest/speedy) encoded byte string.

### `to_bincode() -> bytes`

Feature `bincode` enabled.

Returns statistics as [bincode](https://docs.rs/bincode/latest/bincode) encoded byte string.

### `to_json() -> str`

Feature `json` enabled.

Returns statistics as [json](https://docs.rs/serde_json/latest/serde_json) encoded string.
