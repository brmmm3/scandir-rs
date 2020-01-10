# The API of submodule ``walk``

## ``Toc``

The ``Toc`` class is the return value of method ``toc``, also of property ``toc`` and class method ``collect`` of class ``Walk``.

### ``Toc`` has collowing class members

- ``dirs`` list of directory names.
- ``files`` list of filenames.
- ``symlinks`` list of symlink names.
- ``other`` list of names of all other entry types.
- ``errors`` list of access errors (list of strings).

## ``toc(root_path: str, sorted: bool = False, skip_hidden: bool = False, max_depth: int = 0)``

Scans directory provided through parameter ``root_path`` and returns a ``Toc`` object. This function is blocking and releases the GIL.

### Parameters

- ``root_path`` is directory to scan. ``~`` is allowed on Unix systems.
- ``sorted`` if ``True`` alphabetically sort results.
- ``skip_hidden`` if ``True`` then ignore all hidden files and directories.
- ``max_depth`` is maximum depth of iteration. If ``0`` then depth limit is disabled.

## ``Walk(root_path: str, sorted: bool = False, skip_hidden: bool = False, max_depth: int = 0, iter_type: int = ITER_TYPE_WALK)``

Creates a class object for more control when reading the directory contents. Useful when the iteration should be doine in background without blocking the application. The class instance initially does nothing. To start the scan either the method ``start`` has to be called or a context has to be created (``with ClassInstance:``). When the context is closed the background thread is stopped.

### Parameters

Same as above but with one additional parameter:

- ``iter_type`` defines type of data returned by iterator.

### Iteration types

- ``ITER_TYPE_TOC`` returned data is a ``Toc`` object.
- ``ITER_TYPE_WALK`` returned data is same as returned by ``os.walk``. This is the default since version 0.7.2.
- ``ITER_TYPE_WALKEXT`` returned data contains additional groups: ``symlinks``, ``other`` and ``errors``.

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
