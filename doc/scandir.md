# The API of submodule ``scandir``

## ``Entries``

The ``Entries`` class is the return value of method ``entries``, also of property ``entries`` and class method ``collect`` of class ``Scandir``.

### ``Entries`` has collowing class members

- ``entries`` list of ``Entry`` objects.
- ``duration`` time taken for the iteration in seconds as float.

## ``Entry``

The ``Entry`` class is a property of the ``Entries`` class.

### ``Entry`` has collowing class members

- ``path`` relative path of the entry.
- ``entry`` result for the entry (either ``DirEntry`` or error string).

## ``DirEntry``

### ``DirEntry`` has collowing class members

- ``is_symlink`` ``True`` is entry is a symbolic link.
- ``is_dir`` ``True`` is entry is a directory.
- ``is_file`` ``True`` is entry is a file.
- ``ctime`` creation time in seconds as float.
- ``mtime`` modification time in seconds as float.
- ``atime`` access time in seconds as float.
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

## ``entries(root_path: str, sorted: bool = False, skip_hidden: bool = False, metadata: bool = False, metadata_ext: bool = False, max_depth: int = 0)``

Scans directory provided through parameter ``root_path`` and returns an ``Entries`` object. This function is blocking and releases the GIL.

### Parameters

- ``root_path`` is directory to scan. ``~`` is allowed on Unix systems.
- ``sorted`` if ``True`` alphabetically sort results.
- ``skip_hidden`` if ``True`` ignore all hidden files and directories.
- ``metadata`` if ``True`` also fetch some metadata.
- ``metadata_ext`` if ``True`` also fetch extended metadata.
- ``max_depth`` is maximum depth of iteration. If ``0`` then depth limit is disabled.

## ``Walk(root_path: str, sorted: bool = False, skip_hidden: bool = False, metadata: bool = False, metadata_ext: bool = False, max_depth: int = 0)``

Creates a class object for more control when reading the directory contents. Useful when the iteration should be doine in background without blocking the application. The class instance initially does nothing. To start the scan either the method ``start`` has to be called or a context has to be created (``with ClassInstance:``). When the context is closed the background thread is stopped.

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
