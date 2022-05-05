# The API of class ``Walk``

## ``Toc``

The ``Toc`` class is the return value of class method ``results`` and ``collect`` of class ``Walk``.

### ``Toc`` has following class members

- ``dirs`` list of directory names.
- ``files`` list of filenames.
- ``symlinks`` list of symlink names.
- ``other`` list of names of all other entry types.
- ``errors`` list of access errors (list of strings).

## ``Walk(root_path: str, sorted: bool = False, skip_hidden: bool = False, max_depth: int = 0, max_file_cnt: int = 0, dir_include: List[str] = None, dir_exclude: List[str] = None, file_include: List[str] = None, file_exclude: List[str] = None, case_sensitive: bool = True, return_type: ReturnType = ReturnType.Base)``

Creates a class instance for calculating statistics. The class instance initially does nothing. To start the scan either the method ``start``  or the method ``collect`` has to be called or a context has to be created (``with Walk(...) as instance:``). When the context is closed the background thread is stopped.

### Parameters

Same as above but with one additional parameter:

- ``root_path`` is directory to scan. ``~`` is allowed on Unix systems.
- ``sorted`` if ``True`` alphabetically sort results.
- ``skip_hidden`` if ``True`` then ignore all hidden files and directories.
- ``max_depth`` is maximum depth of iteration. If ``0`` then depth limit is disabled.
- ``dir_include`` list of patterns for directories to include.
- ``dir_exclude`` list of patterns for directories to exclude.
- ``file_include`` list of patterns for files to include.
- ``file_exclude`` list of patterns for files to exclude.
- ``case_sensitive`` if `True` then do case sensitive pattern matching.
- ``return_type`` defines type of data returned by iterator.

For valid file patterns see module [glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

### Iteration types

- ``ReturnType.Base`` return ``dirs`` and ``files`` as ``os.walk`` does.
- ``ReturnType.Ext`` return additional data: ``symlinks``, ``other`` and ``errors``.

**Please note:**
> Due to limitations of jwalk the returned errors just contain the error message without any information to which files the errors correspond to.

### ``toc``

Returns a ``Toc`` object with the current results. The internal results object will be cleared after this call.

### ``has_results()``

Returns ``True`` after iteration has been finished.

### ``as_dict()``

Returns statistics as a ``dict``. Result will only contains the keys of which the values are non zero. The internal results object will be cleared after this call.

### ``collect()``

This does the same as the call of the ``count`` method. It returns a ``Toc`` object and in addition the results are available also within the class instance through the ``toc`` property. This method is blocking and releases the GIL.

### ``start()``

Start iterating through the directory in background.

### ``stop()``

Stop iterating.

### ``busy()``

Return ``True`` when the iteration thread is running.
