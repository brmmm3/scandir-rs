# -*- coding: utf-8 -*-

import os
import time
import tempfile

import pytest
from scandir_rs import Count, Walk, Scandir, ReturnType


def CreateTempFileTree(dircnt: int, depth: int, filecnt: int):
    print(
        f"Create temporary directory with {dircnt} directories with depth {depth} and {3 * filecnt} files")
    tempDir = tempfile.TemporaryDirectory(prefix="scandir_rs_")
    for dn in range(dircnt):
        dirName = f"{tempDir.name}/dir{dn}"
        for depth in range(depth):
            os.makedirs(dirName)
            for fn in range(filecnt):
                open(f"{dirName}/file{fn}.bin", "wb").close()
                open(f"{dirName}/file{fn}.txt", "wb").close()
                open(f"{dirName}/file{fn}.log", "wb").close()
            dirName = f"{dirName}/dir{depth}"
    return tempDir


@pytest.fixture(scope="session", autouse=True)
def tempDir():
    tmpDir = CreateTempFileTree(10, 3, 10)
    yield tmpDir
    tmpDir.cleanup()


def test_count(tempDir):
    count = Count(tempDir.name)
    count.start()
    count.join()
    assert count.as_dict() == {'dirs': 6, 'files': 180,
                               'size': 24576, 'usage': 24576}


def test_count_extended(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext).collect()
    assert count.as_dict() == {'dirs': 6, 'files': 180,
                               'size': 24576, 'usage': 24576}


def test_count_extended_file_exclude(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  file_exclude=["*.bin"]).collect()
    assert count.as_dict() == {'dirs': 6, 'files': 120,
                               'size': 24576, 'usage': 24576}


def test_count_extended_file_include(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  file_include=["*.bin"]).collect()
    assert count.as_dict() == {'dirs': 6, 'files': 60,
                               'size': 24576, 'usage': 24576}


def test_count_extended_dir_include(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  dir_include=["dir0/**"]).collect()
    assert count.as_dict() == {'dirs': 3, 'files': 90,
                               'size': 12288, 'usage': 12288}


def test_count_extended_dir_exclude(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  dir_exclude=["dir0", "dir1"]).collect()
    assert count.as_dict() == {'dirs': 1, 'files': 30,
                               'size': 4096, 'usage': 4096}


def test_walk_toc(tempDir):
    sd = Walk(tempDir.name, return_type=ReturnType.Walk)
    toc = sd.collect()
    assert not toc.errors
    assert not toc.other
    assert not toc.symlinks
    assert len(toc.dirs) == 6
    assert len(toc.files) == 180


def test_walk_toc_iter(tempDir):
    sd = Walk(tempDir.name, return_type=ReturnType.Base)
    sd.start()
    while sd.busy():
        time.sleep(0.01)
    toc = sd.collect()
    assert not toc.errors
    assert not toc.other
    assert not toc.symlinks
    assert len(toc.dirs) == 6
    assert len(toc.files) == 180


def test_walk_walk(tempDir):
    sd = Walk(tempDir.name, return_type=ReturnType.Walk)
    allDirs = []
    allFiles = []
    for root, dirs, files in sd:
        allDirs.extend(dirs)
        allFiles.extend(files)
    assert len(allDirs) == 6
    assert len(allFiles) == 180


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
    assert len(allDirs) == 6
    assert len(allFiles) == 180


def test_scandir_invalid(tempDir):
    with pytest.raises(Exception) as exc:
        instance = Scandir(tempDir.name, return_type=ReturnType.Walk)
        instance.start()
    assert "Parameter return_type has invalid value" in str(exc.value)


def test_scandir_fast(tempDir):
    sd = Scandir(tempDir.name, return_type=ReturnType.Fast)
    contents = {}
    for dirEntry in sd:
        assert dirEntry.st_atime > 0.0
        assert dirEntry.st_ctime > 0.0
        assert dirEntry.st_mtime > 0.0
        assert not hasattr(dirEntry, "st_mode")
        contents[dirEntry.path] = dirEntry
    assert len(contents) == 186


def test_scandir_ext(tempDir):
    sd = Scandir(tempDir.name, return_type=ReturnType.Ext)
    contents = {}
    for dirEntry in sd:
        assert dirEntry.st_atime > 0.0
        assert dirEntry.st_ctime > 0.0
        assert dirEntry.st_mtime > 0.0
        assert hasattr(dirEntry, "st_mode")
        contents[dirEntry.path] = dirEntry
    assert len(contents) == 186
