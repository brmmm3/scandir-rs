"""Type stubs for ``scandir_rs``."""

from collections.abc import Iterator
from datetime import datetime
from enum import Enum
from typing import Any, Self, TypeAlias

__version__: str


class ReturnType(str, Enum):
    """Return type selector for scandir and walk operations."""

    Base = "Base"
    Ext = "Ext"


class DirEntry:
    """Minimal directory entry with basic file metadata."""

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
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...


class DirEntryExt(DirEntry):
    """Directory entry with extended stat metadata."""

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
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...


Entry: TypeAlias = DirEntry | DirEntryExt
WalkItem: TypeAlias = (
    tuple[str, list[str], list[str]]
    | tuple[str, list[str], list[str], list[str], list[str], list[str]]
)


class ScandirResult:
    """Metadata and optional error information for one scan result."""

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

    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...


class ScandirResults:
    """Collected entries and errors from a ``Scandir`` operation."""

    results: list[Entry]
    dirs: list[Entry]
    files: list[Entry]
    symlinks: list[Entry]
    other: list[Entry]
    errors: list[tuple[str, str]]

    def as_dict(self) -> dict[str, Entry | str]: ...
    def __len__(self) -> int: ...
    def __bool__(self) -> bool: ...


class Statistics:
    """Aggregate statistics returned by ``Count`` and ``Scandir``."""

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
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...


class Toc:
    """Grouped file and directory names from a ``Walk`` operation."""

    dirs: list[str]
    files: list[str]
    symlinks: list[str]
    other: list[str]
    errors: list[str]

    def as_dict(self) -> dict[str, Any]: ...
    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...


class Scandir:
    """Asynchronous directory scanner yielding entries and errors."""

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

    def extended(self, extended: bool) -> None: ...
    def clear(self) -> None: ...
    def start(self) -> None: ...
    def join(self) -> bool: ...
    def stop(self) -> bool: ...
    def collect(self, timeout: float | None = None) -> ScandirResults | None: ...
    def has_results(self, only_new: bool | None = None) -> bool: ...
    def results_cnt(self, only_new: bool | None = None) -> int: ...
    def results(self, only_new: bool | None = None) -> ScandirResults: ...
    def has_entries(self, only_new: bool | None = None) -> bool: ...
    def entries_cnt(self, only_new: bool | None = None) -> int: ...
    def entries(self, only_new: bool | None = None) -> list[Entry]: ...
    def has_errors(self) -> bool: ...
    def errors_cnt(self) -> int: ...
    def errors(self, only_new: bool | None = None) -> list[tuple[str, str]]: ...
    def as_dict(self, only_new: bool | None = None) -> dict[str, Entry | str]: ...

    statistics: Statistics
    duration: float
    finished: bool
    busy: bool

    def __enter__(self) -> Self: ...
    def __exit__(
        self,
        ty: type[BaseException] | None = None,
        value: BaseException | None = None,
        traceback: object = None,
    ) -> bool: ...
    def __iter__(self) -> Iterator[Entry | tuple[str, str]]: ...

    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...


class Walk:
    """Recursive directory walker yielding grouped directory entries."""

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

    def extended(self, extended: bool) -> None: ...
    def clear(self) -> None: ...
    def start(self) -> None: ...
    def join(self) -> bool: ...
    def stop(self) -> bool: ...
    def collect(self, timeout: float | None = None) -> Toc | None: ...
    def has_results(self, only_new: bool | None = None) -> bool: ...
    def results_cnt(self, only_new: bool | None = None) -> int: ...
    def results(self, only_new: bool | None = None) -> list[tuple[str, Toc]]: ...
    def has_errors(self) -> bool: ...
    def errors_cnt(self) -> int: ...
    def errors(self, only_new: bool | None = None) -> list[tuple[str, str]]: ...

    statistics: Statistics
    duration: float
    finished: bool
    busy: bool

    def __enter__(self) -> Self: ...
    def __exit__(
        self,
        ty: type[BaseException] | None = None,
        value: BaseException | None = None,
        traceback: object = None,
    ) -> bool: ...
    def __iter__(self) -> Iterator[WalkItem]: ...

    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...


class Count:
    """Directory counter that collects file and directory statistics."""

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

    def extended(self, extended: bool) -> None: ...
    def clear(self) -> None: ...
    def start(self) -> None: ...
    def join(self) -> bool: ...
    def stop(self) -> bool: ...
    def collect(self, timeout: float | None = None) -> Statistics | None: ...
    def has_results(self) -> bool: ...
    def results(self) -> Statistics: ...
    def has_errors(self) -> bool: ...
    def as_dict(self, duration: bool | None = None) -> dict[str, Any]: ...

    duration: float
    finished: bool
    busy: bool

    def __enter__(self) -> Self: ...
    def __exit__(
        self,
        ty: type[BaseException] | None = None,
        value: BaseException | None = None,
        traceback: object = None,
    ) -> bool: ...
    def __iter__(self) -> Iterator[Statistics]: ...

    def to_speedy(self) -> bytes: ...
    def to_bincode(self) -> bytes: ...
    def to_json(self) -> str: ...