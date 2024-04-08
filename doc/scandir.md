# The API of class ``Scandir``

## ``ScandirResult``

Is an enum which can be:

``DirEntry``
``DirEntryExt``

## ``DirEntry``

- ``path`` relative path
- ``is_symlink`` ``True`` is entry is a symbolic link.
- ``is_dir`` ``True`` is entry is a directory.
- ``is_file`` ``True`` is entry is a file.
- ``st_ctime`` creation time in seconds as float.
- ``st_mtime`` modification time in seconds as float.
- ``st_atime`` access time in seconds as float.
- ``st_size`` size of entry.

## ``DirEntryExt``

- ``is_symlink`` ``True`` is entry is a symbolic link.
- ``is_dir`` ``True`` is entry is a directory.
- ``is_file`` ``True`` is entry is a file.
- ``st_ctime`` creation time in seconds as float.
- ``st_mtime`` modification time in seconds as float.
- ``st_atime`` access time in seconds as float.
- ``st_mode`` file access mode / rights.
- ``st_ino`` inode number (only for Unix).
- ``st_dev`` device number (only for Unix).
- ``st_nlink`` number of hard links.
- ``st_size`` size of entry.
- ``st_blksize`` block size of file system.
- ``st_blocks`` number of blocks used.
- ``st_uid`` user id (only for Unix).
- ``st_gid`` groud id (only for Unix).
- ``st_rdev`` device number (for character and block devices on Unix).

## ``Scandir(root_path: str, sorted: bool = False, skip_hidden: bool = False, metadata: bool = False, metadata_ext: bool = False, max_depth: int = 0, dir_include: list = None, dir_exclude: list = None, file_include: list = None, file_exclude: list = None, case_sensitive: bool = True, return_type: int = RETURN_TYPE_WALK, store: bool = true)``

Creates a class object for more control when reading the directory contents. Useful when the iteration should be doine in background without blocking the application. The class instance initially does nothing. To start the scan either the method ``start`` has to be called or a context has to be created (``with ClassInstance:``). When the context is closed the background thread is stopped.

The returned results are tuples with absolute path and `DirEntry`, `DirEntryExt` or `DirEntryFull` object, depending on the `return_type`. In case of an error an error string is returned.

### Parameters

- ``root_path`` is directory to scan. ``~`` is allowed on Unix systems.
- ``sorted`` if ``True`` alphabetically sort results.
- ``skip_hidden`` if ``True`` ignore all hidden files and directories.
- ``metadata`` if ``True`` also fetch some metadata.
- ``metadata_ext`` if ``True`` also fetch extended metadata.
- ``max_depth`` is maximum depth of iteration. If ``0`` then depth limit is disabled.
- ``dir_include`` list of patterns for directories to include.
- ``dir_exclude`` list of patterns for directories to exclude.
- ``file_include`` list of patterns for files to include.
- ``file_exclude`` list of patterns for files to exclude.
- ``case_sensitive`` if `True` then do case sensitive pattern matching.
- ``return_type`` defines type of data returned.
- ``store`` store results in local structure.

For valid file patterns see module [glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

### Return types

- ``ReturnType.Base`` return ``DirEntry`` objects.
- ``ReturnType.Ext`` return ``DirEntryExt`` objects.

### ``clear()``

Clear all results.

### ``start()``

Start parsing the directory tree in background. Raises an expception if a task is already running.

### ``join()``

Wait for parsing task to finish.

### ``stop()``

Stop parsing task.

### ``collect() -> Tuple[List[ScandirResult], List[Tuple[str, str]]]``

Parse file tree and wait until parsing has finished. Method ``start`` will be called if not already done. This method returns the same as the ``results`` method. It is blocking and releases the GIL.
``Error`` contains a tuple with 2 strings. First string contains path to file. Second string is the error message.

### ``has_results(only_new: bool | None = True) -> bool``

Returns ``True`` if new entries or errors are available and ``only_new`` is ``True`` (default) or in case ``only_new`` is ``False`` and any entries and errors have been collected since the start of the parse task.

### ``results_cnt(only_new: bool | None = True) -> int``

Returns the number of new entries and errors if ``only_new`` is ``True`` (default) or in case ``only_new`` is ``False`` the number of entries and errors since the start of the parse task.

### ``results(only_new: bool | None = True) -> Tuple[List[ScandirResult], List[str, str]]``

Returns entries and errors.

If ``only_new`` is ``True`` (default) then return all results and errors collected so far else return only new results and errors.

### ``has_entries(only_new: bool | None = True) -> bool``

Returns ``True`` if new entries are available and ``only_new`` is ``True`` (default) or in case ``only_new`` is ``False`` and any entries have been collected since the start of the parse task.

### ``entries_cnt(only_new: bool | None = True) -> int``

Returns the number of new entries if ``only_new`` is ``True`` (default) or in case ``only_new`` is ``False`` the number of entries since the start of the parse task.

### ``entries(only_new: bool | None = True) -> List[Tuple[str, Toc]]``

Returns entries.

If ``only_new`` is ``True`` (default) then return all results and errors collected so far else return only new results and errors.

### ``has_errors() -> bool``

Returns ``True`` if new errors are available and ``only_new`` is ``True`` (default) or in case ``only_new`` is ``False`` and any errors have been collected since the start of the parse task.

### ``errors_cnt(only_new: bool | None = True) -> int``

Returns the number of new errors if ``only_new`` is ``True`` (default) or in case ``only_new`` is ``False`` the number of errors since the start of the parse task.

### ``errors(only_new: bool | None = True) -> List[Tuple[str, str]]``

Returns errors.

If ``only_new`` is ``True`` (default) then return all results and errors collected so far else return only new results and errors.

### ``as_dict(only_new: bool | None = True) -> Dict[str, DirEntry | DirEntryExt | str]``

Returns entries and errors as dictionary.

If ``only_new`` is ``True`` then return all results collected so far else return only new results. Each result consists of root directory and ``Toc``.

### ``duration() -> float``

Returns the duration of the parsing task. As long as the task is running it will return 0.

### ``finished() -> bool``

Returns ``True`` after the parsing task has finished.

### ``busy()``

Returns ``True`` while a parsing task is running.
