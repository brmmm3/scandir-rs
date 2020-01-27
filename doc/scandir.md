# The API of submodule ``scandir``

## ``Entries``

The ``Entries`` class is the return value of method ``entries``, also of property ``entries`` and class method ``collect`` of class ``Scandir``.

### ``Entries`` has following class members

- ``entries`` list of ``Entry`` objects.
- ``duration`` time taken for the iteration in seconds as float.

## ``Entry``

The ``Entry`` class is a property of the ``Entries`` class.

### ``Entry`` has following class members

- ``path`` relative path of the entry.
- ``entry`` result for the entry (either ``DirEntry`` or error string).

## ``DirEntry``

### ``DirEntry`` has following class members

- ``is_symlink`` ``True`` is entry is a symbolic link.
- ``is_dir`` ``True`` is entry is a directory.
- ``is_file`` ``True`` is entry is a file.
- ``ctime`` creation time in seconds as float.
- ``mtime`` modification time in seconds as float.
- ``atime`` access time in seconds as float.

## ``DirEntryExt``

### ``DirEntryExt`` has following additional class members

- ``mode`` file access mode.
- ``ino`` inode number (only for Unix).
- ``dev`` device number (only for Unix).
- ``nlink`` number of hard links.
- ``size`` size of entry.
- ``blksize`` block size of file system.
- ``blocks`` number of blocks used.
- ``uid`` user id (only for Unix).
- ``gid`` groud id (only for Unix).
- ``rdev`` device number (for character and block devices on Unix).

## ``DirEntryFull``

### ``DirEntryFull`` has following additional class members

- ``name`` filename
- ``path`` relative path

## ``entries(root_path: str, sorted: bool = False, skip_hidden: bool = False, metadata: bool = False, metadata_ext: bool = False, max_depth: int = 0, dir_include: list = None, dir_exclude: list = None, file_include: list = None, file_exclude: list = None, case_sensitive: bool = True)``

Scans directory provided through parameter ``root_path`` and returns an ``Entries`` object. This function is blocking and releases the GIL.

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

For valid file patterns see module [glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

## ``Scandir(root_path: str, sorted: bool = False, skip_hidden: bool = False, metadata: bool = False, metadata_ext: bool = False, max_depth: int = 0, dir_include: list = None, dir_exclude: list = None, file_include: list = None, file_exclude: list = None, case_sensitive: bool = True, return_type: int = RETURN_TYPE_WALK)``

Creates a class object for more control when reading the directory contents. Useful when the iteration should be doine in background without blocking the application. The class instance initially does nothing. To start the scan either the method ``start`` has to be called or a context has to be created (``with ClassInstance:``). When the context is closed the background thread is stopped.

The returned results are tuples with absolute path and `DirEntry`, `DirEntryExt` or `DirEntryFull` object, depending on the `return_type`.

### Parameters

Same as above but with one additional parameter:

- ``return_type`` defines type of data returned by iterator.

### Iteration types

- ``RETURN_TYPE_FAST`` returned data is a ``DirEntry`` object. On Windows ``DirEntry`` doesn't contain valid values!
- ``RETURN_TYPE_BASE`` returned data is a ``DirEntry`` object.
- ``RETURN_TYPE_EXT`` returned data is a ``DirEntryExt`` object.
- ``RETURN_TYPE_FULL`` returned data is a ``DirEntryFull`` object.


### ``entries``

Returns an ``Entries`` object with the current results. The internal results object will be cleared after this call.

### ``has_results()``

Returns ``True`` after iteration has been finished.

### ``as_dict()``

Returns statistics as a ``dict``. Result will only contains the keys of which the values are non zero. The internal results object will be cleared after this call.

### ``collect()``

This does the same as the call of the ``entries`` method. It returns an ``Entries`` object and in addition the results are available also within the class instance through the ``entries`` property. This method is blocking and releases the GIL.

### ``start()``

Start iterating through the directory in background.

### ``stop()``

Stop iterating.

### ``busy()``

Return ``True`` when the iteration thread is running.
