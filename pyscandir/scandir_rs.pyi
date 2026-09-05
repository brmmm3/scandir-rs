"""Type stubs for scandin_rs — a fast file tree scanner written in Rust."""


from collections.abc import Iterator
from datetime import datetime
from enum import Enum
from typing import Any, Self

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
    st_ctime: datetime | None
    st_mtime: datetime | None
    st_atime: datetime | None
    st_size: int
    ctime: float
    mtime: float
    atime: float

    def as_dict(self) -> dict[str, Any]: ...

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

    def as_dict(self) -> dict[str, Any]: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

class ScandirResult:
    """Result item from Scandir — either a DirEntry / DirEntryExt or an error."""

    path: str
    error: tuple[str, str] | None
    is_dir: bool
    is_file: bool
    is_symlink: bool
    ctime: float
    mtime: float
    atime: float
    size: int
    ext: DirEntryExt | None


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
    errors: list[str]
    duration: float

    def as_dict(self, duration: bool | None = None) -> dict[str, Any]: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...

class Toc:
    """Tree-of-contents: grouped file/directory paths from a Walk operation."""

    dirs: list[str]
    files: list[str]
    symlinks: list[str]
    other: list[str]
    errors: list[str]

    def as_dict(self) -> dict[str, Any]: ...

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
        sorted: bool | None = None,
        skip_hidden: bool | None = None,
        max_depth: int | None = None,
        max_file_cnt: int | None = None,
        dir_include: list[str] | None = None,
        dir_exclude: list[str] | None = None,
        file_include: list[str] | None = None,
        file_exclude: list[str] | None = None,
        case_sensitive: bool | None = None,
        follow_links: bool | None = None,
        return_type: ReturnType | None = None,
        store: bool | None = None,
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
    ) -> tuple[list[DirEntry | DirEntryExt], list[tuple[str, str]]]: ...
    def has_results(self, only_new: bool | None = None) -> bool: ...
    def results_cnt(self, only_new: bool | None = None) -> int: ...
    def results(
        self, only_new: bool | None = None
    ) -> tuple[list[DirEntry | DirEntryExt], list[tuple[str, str]]]: ...
    def has_entries(self, only_new: bool | None = None) -> bool: ...
    def entries_cnt(self, only_new: bool | None = None) -> int: ...
    def entries(
        self, only_new: bool | None = None
    ) -> list[DirEntry | DirEntryExt]: ...
    def has_errors(self) -> bool: ...
    def errors_cnt(self) -> int: ...
    def errors(self, only_new: bool | None = None) -> list[tuple[str, str]]: ...
    def as_dict(self, only_new: bool | None = None) -> dict[str, Any]: ...

    # Properties
    statistics: Statistics
    duration: float
    finished: bool
    busy: bool

    # Iterator protocol
    def __iter__(self) -> Iterator[DirEntry | DirEntryExt]: ...

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
        sorted: bool | None = None,
        skip_hidden: bool | None = None,
        max_depth: int | None = None,
        max_file_cnt: int | None = None,
        dir_include: list[str] | None = None,
        dir_exclude: list[str] | None = None,
        file_include: list[str] | None = None,
        file_exclude: list[str] | None = None,
        case_sensitive: bool | None = None,
        follow_links: bool | None = None,
        return_type: ReturnType | None = None,
        store: bool | None = None,
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
    def has_results(self, only_new: bool | None = None) -> bool: ...
    def results_cnt(self, only_new: bool | None = None) -> int: ...
    def results(self, only_new: bool | None = None) -> list[tuple[str, Toc]]: ...
    def has_errors(self) -> bool: ...
    def errors_cnt(self) -> int: ...
    def errors(self, only_new: bool | None = None) -> list[tuple[str, str]]: ...

    # Properties
    statistics: Statistics
    duration: float
    finished: bool

    # Iterator protocol
    def __iter__(self) -> Iterator[tuple[str, list[str], list[str]]]: ...

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
        skip_hidden: bool | None = None,
        max_depth: int | None = None,
        max_file_cnt: int | None = None,
        dir_include: list[str] | None = None,
        dir_exclude: list[str] | None = None,
        file_include: list[str] | None = None,
        file_exclude: list[str] | None = None,
        case_sensitive: bool | None = None,
        follow_links: bool | None = None,
        return_type: ReturnType | None = None,
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
    def as_dict(self, duration: bool | None = None) -> dict[str, Any]: ...

    # Properties
    duration: float
    finished: bool
    busy: bool

    # Context-manager protocol
    def __enter__(self) -> Self: ...
    def __exit__(
        self,
        ty: type[BaseException] | None,
        value: BaseException | None,
        traceback: object,
    ) -> None: ...

    # Feature-gated serialization
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...
