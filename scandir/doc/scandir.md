# The API of class `Scandir`

## ScandirResult

Is an enum which can be:

`DirEntry`
`DirEntryExt`

## DirEntry

- `path` relative path
- `is_symlink` `True` is entry is a symbolic link.
- `is_dir` `True` is entry is a directory.
- `is_file` `True` is entry is a file.
- `st_ctime` creation time in seconds as float.
- `st_mtime` modification time in seconds as float.
- `st_atime` access time in seconds as float.
- `st_size` size of entry.

## DirEntryExt

- `is_symlink` `True` is entry is a symbolic link.
- `is_dir` `True` is entry is a directory.
- `is_file` `True` is entry is a file.
- `st_ctime` creation time in seconds as float.
- `st_mtime` modification time in seconds as float.
- `st_atime` access time in seconds as float.
- `st_mode` file access mode / rights.
- `st_ino` inode number (only for Unix).
- `st_dev` device number (only for Unix).
- `st_nlink` number of hard links.
- `st_size` size of entry.
- `st_blksize` block size of file system.
- `st_blocks` number of blocks used.
- `st_uid` user id (only for Unix).
- `st_gid` groud id (only for Unix).
- `st_rdev` device number (for character and block devices on Unix).

## `Scandir::new<P: AsRef<Path>>(root_path: P, store: Option<bool>) -> Result<Self, Error>`

Creates a class instance for getting the metadata of the entries of a file tree.
The class instance initially does nothing. To start the scan either the method `start`
or the method `collect` has to be called.

### Class members

- `root_path` is directory to scan. `~` is allowed on Unix systems.
- `sorted` if `true` alphabetically sort results.
- `skip_hidden` if `true` ignore all hidden files and directories.
- `metadata` if `true` also fetch some metadata.
- `metadata_ext` if `true` also fetch extended metadata.
- `max_depth` is maximum depth of iteration. If `0` then depth limit is disabled.
- `dir_include` list of patterns for directories to include.
- `dir_exclude` list of patterns for directories to exclude.
- `file_include` list of patterns for files to include.
- `file_exclude` list of patterns for files to exclude.
- `case_sensitive` if `true` then do case sensitive pattern matching.
- `return_type` defines type of data returned.
- `store` store results in local structure.

For valid file patterns see module [glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

### Return types

- `ReturnType::Base` return `DirEntry` objects.
- `ReturnType::Ext` return `DirEntryExt` objects.

### `sorted(mut self, sorted: bool) -> Self`

Return results in sorted order.

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

### `return_type(mut self, return_type: ReturnType) -> Self`

Set extended file type counting.

### `clear(&mut self)`

Clear all results.

### `start(&mut self) -> Result<(), Error>`

Start parsing the directory tree in background. Raises an exception if a task is already running.

### `join(&mut self) -> bool`

Wait for parsing task to finish.

### `stop(&mut self) -> bool`

Stop parsing task.

### `collect(&mut self) -> Result<ScandirResults, Error>`

Calculate statistics and return a `Toc` object when the task has finished. This method is blocking.

### `has_results(&mut self, only_new: bool) -> bool`

If `only_new` is `true` this method returns `true` if new results are available,
If `only_new` is `false` this method returns `true` if results are available,

### `results_cnt(&mut self, only_new: bool) -> usize`

If `only_new` is `true` this method returns number of new results,
If `only_new` is `false` this method returns number of total results,

### `results(&mut self, only_new: bool) -> ScandirResults`

If `only_new` is `true` this method returns new results,
If `only_new` is `false` this method returns total results,

### `has_entries(&mut self, only_new: bool) -> bool`

If `only_new` is `true` this method returns `true` if new results are available,
If `only_new` is `false` this method returns `true` if results are available,

### `entries_cnt(&mut self, only_new: bool) -> usize`

If `only_new` is `true` this method returns number of new results,
If `only_new` is `false` this method returns number of total results,

### `entries(&mut self, only_new: bool) -> Vec<ScandirResult>`

If `only_new` is `true` this method returns new results,
If `only_new` is `false` this method returns total results,

### `has_errors(&mut self) -> bool`

Returns `true` if errors occured while scanning the file tree.

### `errors_cnt(&mut self) -> usize`

Returns number of errors occured while scanning the file tree.

### `errors(&mut self, only_new: bool) -> ErrorsType`

Returns the errors.

### `to_speedy(&self) -> Result<Vec<u8>, speedy::Error>`

Returns the results serialized with `speedy`.
For this method the feature `speedy` needs to be enabled.

### `to_bincode(&self) -> bincode::Result<Vec<u8>>`

Returns the results serialized with `bincode`.
For this method the feature `bincode` needs to be enabled.

### `to_json(&self) -> serde_json::Result<String>`

Returns the results serialized as `json`.
For this method the feature `json` needs to be enabled.

### `statistics(&self) -> Statistics`

Returns the statistics of the results.

### `duration(&mut self) -> f64`

Returns the duration of the task in seconds as float. As long as the task is running it will
return 0.

### `finished(&self) -> bool`

Returns `true` after the task has finished.

### `busy(&self) -> bool`

Returns `true` while a task is running.
