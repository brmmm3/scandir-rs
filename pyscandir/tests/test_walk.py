
import time

import pytest

from scandir_rs import ReturnType, Walk

from .common import CreateTempFileTree


@pytest.fixture(scope="session", autouse=True)
def tempDir():
    tmpDir = CreateTempFileTree(10, 3, 10)
    yield tmpDir
    tmpDir.cleanup()


def test_walk_toc(tempDir):
    sd = Walk(tempDir.name)
    toc = sd.collect()
    assert not toc.errors
    assert not toc.other
    assert not toc.symlinks
    import os

    print(f"tempDir.name={tempDir.name}")
    print(f"toc.dirs={toc.dirs}")
    print(f"toc.files={toc.files}")
    print(f"entries={os.listdir(tempDir.name)}")
    assert len(toc.dirs) == 30
    assert len(toc.files) == 900


def test_walk_toc_extended(tempDir):
    sd = Walk(tempDir.name, return_type=ReturnType.Ext)
    toc = sd.collect()
    assert not toc.errors
    assert not toc.other
    assert not toc.symlinks
    import os

    print(f"tempDir.name={tempDir.name}")
    print(f"toc.dirs={toc.dirs}")
    print(f"toc.files={toc.files}")
    print(f"entries={os.listdir(tempDir.name)}")
    assert len(toc.dirs) == 30
    assert len(toc.files) == 900


def test_walk_toc_iter(tempDir):
    sd = Walk(tempDir.name, return_type=ReturnType.Base)
    sd.start()
    while sd.busy:
        time.sleep(0.01)
    toc = sd.collect()
    assert not toc.errors
    assert not toc.other
    assert not toc.symlinks
    assert len(toc.dirs) == 30
    assert len(toc.files) == 900


def test_walk_walk(tempDir):
    sd = Walk(tempDir.name, return_type=ReturnType.Base)
    allDirs = []
    allFiles = []
    for root, dirs, files in sd:
        allDirs.extend(dirs)
        allFiles.extend(files)
    assert len(allDirs) == 30
    assert len(allFiles) == 900


def test_walk_walk_ext(tempDir):
    sd = Walk(tempDir.name, return_type=ReturnType.Ext)
    allDirs = []
    allFiles = []
    allSymlinks = []
    allOther = []
    allErrors = []
    for root, dirs, files, symlinks, other, errors in sd:
        allDirs.extend(dirs)
        allFiles.extend(files)
        allSymlinks.extend(symlinks)
        allOther.extend(other)
        allErrors.extend(errors)
    assert not allErrors
    assert not allOther
    assert not allSymlinks
    assert len(allDirs) == 30
    assert len(allFiles) == 900


def test_walk_relative_directory_include_matches_root_child(tmp_path):
    (tmp_path / "included" / "nested").mkdir(parents=True)
    (tmp_path / "excluded").mkdir()
    (tmp_path / "included" / "nested" / "kept.txt").touch()
    (tmp_path / "excluded" / "omitted.txt").touch()

    toc = Walk(str(tmp_path), dir_include=["included/**"]).collect()

    assert toc is not None
    assert set(toc.dirs) == {"included", "included/nested"}
    assert toc.files == ["included/nested/kept.txt"]


def test_walk_context_manager_does_not_suppress_value_error(tmp_path):
    with pytest.raises(ValueError, match="caller error"):
        with Walk(str(tmp_path)):
            raise ValueError("caller error")


def test_walk_iter_returns_active_iterator_unchanged(tmp_path):
    (tmp_path / "entry.txt").touch()
    iterator = iter(Walk(str(tmp_path)))

    assert iter(iterator) is iterator
    assert [files for _, _, files in iterator] == [["entry.txt"]]


@pytest.mark.parametrize("timeout", [-1.0, float("nan"), float("inf"), 1e19, 1e300])
def test_walk_rejects_invalid_collect_timeout(tmp_path, timeout):
    with pytest.raises(ValueError, match="timeout"):
        Walk(str(tmp_path)).collect(timeout=timeout)


@pytest.mark.parametrize("store", [False, True])
def test_walk_collect_timeout_returns_toc(tmp_path, store):
    (tmp_path / "entry.txt").touch()

    toc = Walk(str(tmp_path), store=store).collect(timeout=10.0)

    assert toc is not None
    assert toc.files == ["entry.txt"]


@pytest.mark.parametrize("store", [False, True])
def test_walk_collect_timeout_retry_preserves_results(tempDir, store):
    walker = Walk(tempDir.name, store=store)
    toc = walker.collect(timeout=1e-9)
    if toc is None:
        toc = walker.collect(timeout=10.0)

    assert toc is not None
    assert not toc.errors
    assert len(toc.dirs) == len(set(toc.dirs)) == 30
    assert len(toc.files) == len(set(toc.files)) == 900
