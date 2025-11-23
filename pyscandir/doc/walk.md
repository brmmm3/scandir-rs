# The API of class `Walk`

## `Toc`

The `Toc` class is the return value of class method `results` and `collect` of class `Walk`.

### `Toc` has following class members

- `dirs` list of directory names.
- `files` list of filenames.
- `symlinks` list of symlink names.
- `other` list of names of all other entry types.
- `errors` list of access errors (list of strings).

## `Walk()`

```python
def Walk(
    root_path: str,
    sorted: bool = False,
    skip_hidden: bool = False,
    max_depth: int = 0,
    max_file_cnt: int = 0,
    dir_include: List[str] | None = None,
    dir_exclude: List[str] | None = None,
    file_include: List[str] | None = None,
    file_exclude: List[str] | None = None,
    case_sensitive: bool = True,
    return_type: ReturnType = ReturnType.Base,
    store: bool = True,
)
```

Creates a class instance for calculating statistics. The class instance initially does nothing.
To start the scan either the method `start`  or the method `collect` has to be called or a context
has to be created (`with Walk(...) as instance:`). When the context is closed the background
thread is stopped.

### Parameters

- `root_path` is directory to scan. `~` is allowed on Unix systems.
- `sorted` if `True` alphabetically sort results.
- `skip_hidden` if `True` then ignore all hidden files and directories.
- `max_depth` is maximum depth of iteration. If `0` then depth limit is disabled.
- `dir_include` list of patterns for directories to include.
- `dir_exclude` list of patterns for directories to exclude.
- `file_include` list of patterns for files to include.
- `file_exclude` list of patterns for files to exclude.
- `case_sensitive` if `True` then do case sensitive pattern matching.
- `follow_links` if `True` then follow symlinks and junctions.
- `return_type` defines type of data returned.
- `store` store results in local structure.

For valid file patterns see module [glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

### Return types

- `ReturnType.Base` return `dirs` and `files` as `os.walk` does.
- `ReturnType.Ext` return additional data: `symlinks`, `other` and `errors`.

**Please note:**
> Due to limitations of jwalk the returned errors just contain the error message without any
information to which files the errors correspond to.

### `clear()`

Clear all results.

### `start()`

Start parsing the directory tree in background. Raises an exception if a task is already running.

### `join()`

Wait for task to finish.

### `stop()`

Stop task.

### `collect() -> Toc`

Collect directories, files, etc. and return a `Toc` object when the task has finished.
This method is blocking and releases the GIL. Method `start` will be called if not already done.

### `has_results(only_new: bool | None = True) -> bool`

Returns `True` if new entries are available and `only_new` is `False` or in case `only_new`
is `False` and any entries have been collected since task start.

### `results_cnt(only_new: bool | None = True) -> int`

Returns number of results collected so far. If `update` is `True` then new results are counted too.

### `results(ronly_new: bool | None = True) -> List[Tuple[str, Toc]]`

Returns entries and errors.

If `only_new` is `True` (default) then return all `Toc` collected so far else return only new `Toc`.

### `has_errors() -> bool`

Returns `True` if errors occured while walking through the directory tree.
The error messages can be found in `Toc` objects returned.

### `duration -> float`

Returns the duration of the task in seconds as float. As long as the task is running it will
return 0.

### `finished -> bool`

Returns `True` after the task has finished.

### `busy -> bool`

Returns `True` while a task is running.
