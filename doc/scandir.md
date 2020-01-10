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

    pub mode: u32,
    pub ino: u64,
    pub dev: u64,
    pub nlink: u64,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,

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
