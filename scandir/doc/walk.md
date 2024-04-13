# The API of class `Walk`

## Toc

The `Toc` class is the return value of class method `results` and `collect` of class `Walk`.

### `Toc` has following class members

- `dirs` list of directory names.
- `files` list of filenames.
- `symlinks` list of symlink names.
- `other` list of names of all other entry types.
- `errors` list of access errors (list of strings).

## Walk::new<P: AsRef<Path>>(root_path: P, store: Option<bool>) -> Result<Self, Error>

Creates a class instance for getting the file tree. The class instance initially does nothing. To start the scan either the method `start` or the method `collect` has to be called.

### Class members

- `root_path` is directory to scan. `~` is allowed on Unix systems.
- `sorted` if `true` alphabetically sort results.
- `skip_hidden` if `true` then ignore all hidden files and directories.
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

- `ReturnType::Base` return `dirs` and `files` as `os.walk` does.
- `ReturnType::Ext` return additional data: `symlinks`, `other` and `errors`.

**Please note:**
> Due to limitations of jwalk the returned errors just contain the error message without any information to which files the errors correspond to.

### sorted(mut self, sorted: bool) -> Self

Return results in sorted order.

### skip_hidden(mut self, skip_hidden: bool) -> Self

Set to `true` to skip hidden (starting with a dot) files.

### max_depth(mut self, depth: usize) -> Self

Set the maximum depth of entries yield by the iterator.

### max_file_cnt(mut self, max_file_cnt: usize) -> Self

Set maximum number of files to collect.

### dir_include(mut self, dir_include: Option<Vec<String>>) -> Self

Set directory include filter.

### dir_exclude(mut self, dir_exclude: Option<Vec<String>>) -> Self

Set directory exclude filter.

### file_include(mut self, file_include: Option<Vec<String>>) -> Self

Set file include filter.

### file_exclude(mut self, file_exclude: Option<Vec<String>>) -> Self

Set file exclude filter.

### case_sensitive(mut self, case_sensitive: bool) -> Self

Set case sensitive filename filtering.

### return_type(mut self, return_type: ReturnType) -> Self

Set extended file type counting.

### clear(&mut self)

Clear all results.

### start(&mut self) -> Result<(), Error>

Start parsing the directory tree in background. Raises an exception if a task is already running.

### join(&mut self) -> bool

Wait for parsing task to finish.

### stop(&mut self) -> bool

Stop parsing task.

### collect(&mut self) -> Result<Toc, Error>

Calculate statistics and return a `Toc` object when the task has finished. This method is blocking.

### has_results(&mut self, only_new: bool) -> bool

If `only_new` is `true` this method returns `true` if new results are available,
If `only_new` is `false` this method returns `true` if results are available,

### results_cnt(&mut self, only_new: bool) -> usize

If `only_new` is `true` this method returns number of new results,
If `only_new` is `false` this method returns number of total results,

### results(&mut self, only_new: bool) -> Vec<(String, Toc)>

If `only_new` is `true` this method returns new results,
If `only_new` is `false` this method returns total results,

### has_errors(&mut self) -> bool

Returns `true` if errors occured while scanning the directory tree. The errors can be found in the statistics object.

### duration(&mut self) -> f64

Returns the duration of the task in seconds as float. As long as the task is running it will return 0.

### finished(&self) -> bool

Returns `true` after the task has finished.

### busy(&self) -> bool

Returns `true` while a task is running.
