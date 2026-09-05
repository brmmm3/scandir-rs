"""Type stubs for scandin_rs — a fast file tree scanner written in Rust."""

from __future__ import annotations

import sys
from datetime import datetime
from enum import Enum
from typing import Any, Dict, Iterator, List, Optional, Tuple, Union

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

__version__: str

# ---------------------------------------------------------------------------
# Enums
# ---------------------------------------------------------------------------

class ReturnType(str, Enum):
    """Return type selector for scandir / walk operations."""

    Base = "Base"
    Ext = "Ext"

# ---------------------------------------------------------------------------
# Data classes
# ---------------------------------------------------------------------------

class DirEntry:
    """Minimal directory entry with path, type flags and basic timestamps."""

    path: str
    is_symlink: bool
    is_dir: bool
    is_file: bool
    st_ctime: Optional[datetime]
    st_mtime: Optional[datetime]
    st_atime: Optional[datetime]
    st_size: int
    ctime: float
    mtime: float
    atime: float

    def as_dict(self) -> Dict[str, Any]: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

class DirEntryExt(DirEntry):
    """Extended directory entry with full POSIX stat fields."""

    st_blksize: int
    st_blocks: int
    st_mode: int
    st_nlink: int
    st_uid: int
    st_gid: int
    st_ino: int
    st_dev: int
    st_rdev: int

    def as_dict(self) -> Dict[str, Any]: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

class ScandirResult:
    """Result item from Scandir — either a DirEntry / DirEntryExt or an error."""

    path: str
    error: Optional[Tuple[str, str]]
    is_dir: bool
    is_file: bool
    is_symlink: bool
    ctime: float
    mtime: float
    atime: float
    size: int
    ext: Optional[DirEntryExt]

    def __repr__(self) -> str: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

class Statistics:
    """Aggregate statistics from a Count or Scandir operation."""

    dirs: int
    files: int
    slinks: int
    hlinks: int
    devices: int
    pipes: int
    size: int
    usage: int
    errors: List[str]
    duration: float

    def as_dict(self, duration: Optional[bool] = None) -> Dict[str, Any]: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

class Toc:
    """Tree-of-contents: grouped file/directory paths from a Walk operation."""

    dirs: List[str]
    files: List[str]
    symlinks: List[str]
    other: List[str]
    errors: List[str]

    def as_dict(self) -> Dict[str, Any]: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

# ---------------------------------------------------------------------------
# Iterator classes
# ---------------------------------------------------------------------------

class Scandir:
    """Async directory scanner that yields DirEntry / DirEntryExt objects.

    Can be used as a simple iterator (blocks until finished) or started
    manually for concurrent / incremental access.
    """

    def __init__(
        self,
        root_path: str,
        sorted: Optional[bool] = None,
        skip_hidden: Optional[bool] = None,
        max_depth: Optional[int] = None,
        max_file_cnt: Optional[int] = None,
        dir_include: Optional[List[str]] = None,
        dir_exclude: Optional[List[str]] = None,
        file_include: Optional[List[str]] = None,
        file_exclude: Optional[List[str]] = None,
        case_sensitive: Optional[bool] = None,
        follow_links: Optional[bool] = None,
        return_type: Optional[ReturnType] = None,
        store: Optional[bool] = None,
    ) -> None: ...

    # Configuration
    def extended(self, extended: bool) -> None: ...
    def clear(self) -> None: ...

    # Lifecycle
    def start(self) -> None: ...
    def join(self) -> bool: ...
    def stop(self) -> bool: ...

    # Data access
    def collect(
        self,
    ) -> Tuple[List[Union[DirEntry, DirEntryExt]], List[Tuple[str, str]]]: ...
    def has_results(self, only_new: Optional[bool] = None) -> bool: ...
    def results_cnt(self, only_new: Optional[bool] = None) -> int: ...
    def results(
        self, only_new: Optional[bool] = None
    ) -> Tuple[List[Union[DirEntry, DirEntryExt]], List[Tuple[str, str]]]: ...
    def has_entries(self, only_new: Optional[bool] = None) -> bool: ...
    def entries_cnt(self, only_new: Optional[bool] = None) -> int: ...
    def entries(
        self, only_new: Optional[bool] = None
    ) -> List[Union[DirEntry, DirEntryExt]]: ...
    def has_errors(self) -> bool: ...
    def errors_cnt(self) -> int: ...
    def errors(self, only_new: Optional[bool] = None) -> List[Tuple[str, str]]: ...
    def as_dict(self, only_new: Optional[bool] = None) -> Dict[str, Any]: ...

    # Properties
    statistics: Statistics
    duration: float
    finished: bool
    busy: bool

    # Iterator protocol
    def __iter__(self) -> Iterator[Union[DirEntry, DirEntryExt]]: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

class Walk:
    """Recursive directory walker yielding (root, dirs, files) tuples.

    Supports optional symlink / other / error groups when return_type=ReturnType.Ext.
    """

    def __init__(
        self,
        root_path: str,
        sorted: Optional[bool] = None,
        skip_hidden: Optional[bool] = None,
        max_depth: Optional[int] = None,
        max_file_cnt: Optional[int] = None,
        dir_include: Optional[List[str]] = None,
        dir_exclude: Optional[List[str]] = None,
        file_include: Optional[List[str]] = None,
        file_exclude: Optional[List[str]] = None,
        case_sensitive: Optional[bool] = None,
        follow_links: Optional[bool] = None,
        return_type: Optional[ReturnType] = None,
        store: Optional[bool] = None,
    ) -> None: ...

    # Configuration
    def extended(self, extended: bool) -> None: ...
    def clear(self) -> None: ...

    # Lifecycle
    def start(self) -> None: ...
    def join(self) -> bool: ...
    def stop(self) -> bool: ...

    # Data access
    def collect(self) -> Toc: ...
    def has_results(self, only_new: Optional[bool] = None) -> bool: ...
    def results_cnt(self, only_new: Optional[bool] = None) -> int: ...
    def results(self, only_new: Optional[bool] = None) -> List[Tuple[str, Toc]]: ...
    def has_errors(self) -> bool: ...
    def errors_cnt(self) -> int: ...
    def errors(self, only_new: Optional[bool] = None) -> List[Tuple[str, str]]: ...

    # Properties
    statistics: Statistics
    duration: float
    finished: bool

    # Iterator protocol
    def __iter__(self) -> Iterator[Tuple[str, List[str], List[str]]]: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

class Count:
    """Directory counter that collects file/directory statistics.

    Supports context-manager protocol (``with Count(path) as c: ...``).
    """

    def __init__(
        self,
        root_path: str,
        skip_hidden: Optional[bool] = None,
        max_depth: Optional[int] = None,
        max_file_cnt: Optional[int] = None,
        dir_include: Optional[List[str]] = None,
        dir_exclude: Optional[List[str]] = None,
        file_include: Optional[List[str]] = None,
        file_exclude: Optional[List[str]] = None,
        case_sensitive: Optional[bool] = None,
        follow_links: Optional[bool] = None,
        return_type: Optional[ReturnType] = None,
    ) -> None: ...

    # Configuration
    def extended(self, extended: bool) -> None: ...
    def clear(self) -> None: ...

    # Lifecycle
    def start(self) -> None: ...
    def join(self) -> bool: ...
    def stop(self) -> bool: ...

    # Data access
    def collect(self) -> Statistics: ...
    def has_results(self) -> bool: ...
    def results(self) -> Statistics: ...
    def has_errors(self) -> bool: ...
    def as_dict(self, duration: Optional[bool] = None) -> Dict[str, Any]: ...

    # Properties
    duration: float
    finished: bool
    busy: bool

    # Context-manager protocol
    def __enter__(self) -> Count: ...
    def __exit__(
        self,
        ty: Optional[type],
        value: Optional[BaseException],
        traceback: Optional[Any],
    ) -> None: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...
