# The API of class `Count`

## Statistics

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

## `Count::new<P: AsRef<Path>>(root_path: P) -> Result<Self, Error>`

Creates a class instance for calculating statistics. The class instance initially does nothing.
To start the scan either the method `start` or the method `collect` has to be called.

### Class members

- `root_path` is directory to scan. `~` is allowed on Unix systems.
- `skip_hidden` if `true` then ignore all hidden files and directories.
- `max_depth` is maximum depth of iteration. If `0` then depth limit is disabled.
- `max_file_cnt` is maximum number of files to collect. If `0` then limit is disabled.
- `dir_include` list of patterns for directories to include.
- `dir_exclude` list of patterns for directories to exclude.
- `file_include` list of patterns for files to include.
- `file_exclude` list of patterns for files to exclude.
- `case_sensitive` if `true` then do case sensitive pattern matching.
- `return_type` defines type of data returned.

For valid file patterns see module [glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

### Return types

- `ReturnType::Base` calculate statistics for `dirs`, `files`, `slinks`, `size` and `usage`.
- `ReturnType::Ext` in addition to above calculate statistcs `hlinks` and on Unix platforms

 `devices` and `pipes`.

### `skip_hidden(mut self, skip_hidden: bool) -> Self`

Set to `true` to skip hidden (starting with a dot) files.

### `max_depth(mut self, depth: usize) -> Self`

Set the maximum depth of entries yield by the iterator.

### `max_file_cnt(mut self, max_file_cnt: usize) -> Self`

Set maximum number of files to collect.

### `dir_include(mut self, dir_include: Option<Vec<String>>) -> Self`

Set directory include filter.

### `dir_exclude(mut self, dir_exclude: Option<Vec<String>>) -> Self`

Set directory exclude filter.

### `file_include(mut self, file_include: Option<Vec<String>>) -> Self`

Set file include filter.

### `file_exclude(mut self, file_exclude: Option<Vec<String>>) -> Self`

Set file exclude filter.

### `case_sensitive(mut self, case_sensitive: bool) -> Self`

Set case sensitive filename filtering.

### `extended(mut self, extended: bool) -> Self`

Set extended file type counting.

### `clear(&mut self)`

Clear all results.

### `start(&mut self) -> Result<(), Error>`

Start calculating statistics in background. Raises an expception if a task is already running.

### `join(&mut self) -> bool`

Wait for parsing task to finish.

### `stop(&mut self) -> bool`

Stop parsing task.

### `collect(&mut self) -> Result<Statistics, Error>`

Calculate statistics and return a `Statistics` object when the task has finished.

### `has_results(&self) -> bool`

Returns `true` if new statistics are available.

### `results(&mut self) -> Statistics`

Return a `Statistics` object with the current statistics.

### `has_errors(&mut self) -> bool`

Returns `true` if errors occured while scanning the directory tree. The errors can be found
 in the statistics object.

### `duration(&mut self) -> f64`

Returns the duration of the task in seconds as float. As long as the task is running it will
return 0.

### `finished(&self) -> bool`

Returns `true` after the task has finished.

### `busy(&self) -> bool`

Returns `true` while a task is running.
