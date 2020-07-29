# -*- coding: utf-8 -*-

import os
import time
import tempfile

import pytest
import scandir_rs as scandir


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
    count = scandir.count.count(tempDir.name)
    assert count.as_dict() == {'dirs': 7, 'files': 180}


def test_count_extended(tempDir):
    count = scandir.count.count(tempDir.name, extended=True)
    assert count.as_dict() == {'dirs': 7, 'files': 180,
                               'size': 28672, 'usage': 28672}


def test_count_extended_file_exclude(tempDir):
    count = scandir.count.count(
        tempDir.name, extended=True, file_exclude=["*.bin"])
    assert count.as_dict() == {'dirs': 7, 'files': 120,
                               'size': 28672, 'usage': 28672}


def test_count_extended_file_include(tempDir):
    count = scandir.count.count(
        tempDir.name, extended=True, file_include=["*.bin"])
    assert count.as_dict() == {'dirs': 7, 'files': 60,
                               'size': 28672, 'usage': 28672}


def test_count_extended_dir_include(tempDir):
    count = scandir.count.count(
        tempDir.name, extended=True, dir_include=["dir0/**"])
    assert count.as_dict() == {'dirs': 4, 'files': 90,
                               'size': 16384, 'usage': 16384}


def test_count_extended_dir_exclude(tempDir):
    count = scandir.count.count(
        tempDir.name, extended=True, dir_exclude=["dir0", "dir1"])
    assert count.as_dict() == {'dirs': 2, 'files': 30,
                               'size': 8192, 'usage': 8192}


def test_walk_toc(tempDir):
    sd = scandir.walk.Walk(tempDir.name,
                           return_type=scandir.RETURN_TYPE_WALK)
    toc = sd.collect()
    assert not toc.errors
    assert not toc.other
    assert not toc.symlinks
    assert len(toc.dirs) == 7
    assert len(toc.files) == 180


def test_walk_toc_iter(tempDir):
    sd = scandir.walk.Walk(tempDir.name,
                           return_type=scandir.RETURN_TYPE_BASE)
    sd.start()
    while sd.busy():
        time.sleep(0.01)
    toc = sd.toc
    assert not toc.errors
    assert not toc.other
    assert not toc.symlinks
    assert len(toc.dirs) == 7
    assert len(toc.files) == 180


def test_walk_walk(tempDir):
    sd = scandir.walk.Walk(tempDir.name,
                           return_type=scandir.RETURN_TYPE_WALK)
    allDirs = []
    allFiles = []
    for root, dirs, files in sd:
        allDirs.extend(dirs)
        allFiles.extend(files)
    assert len(allDirs) == 7
    assert len(allFiles) == 180


def test_walk_walk_ext(tempDir):
    sd = scandir.walk.Walk(tempDir.name,
                           return_type=scandir.RETURN_TYPE_EXT)
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
    assert len(allDirs) == 7
    assert len(allFiles) == 180


def test_scandir_invalid(tempDir):
    with pytest.raises(Exception) as exc:
        scandir.scandir.Scandir(tempDir.name,
                                return_type=scandir.RETURN_TYPE_WALK)
    assert "Parameter return_type has invalid value" in str(exc.value)


def test_scandir_fast(tempDir):
    sd = scandir.scandir.Scandir(tempDir.name,
                                 return_type=scandir.RETURN_TYPE_FAST)
    contents = {}
    for pathName, dirEntry in sd:
        assert dirEntry.st_atime > 0.0
        assert dirEntry.st_ctime > 0.0
        assert dirEntry.st_mtime > 0.0
        assert not hasattr(dirEntry, "st_mode")
        contents[pathName] = dirEntry
    assert len(contents) == 187


def test_scandir_ext(tempDir):
    sd = scandir.scandir.Scandir(tempDir.name,
                                 return_type=scandir.RETURN_TYPE_EXT)
    contents = {}
    for pathName, dirEntry in sd:
        assert dirEntry.st_atime > 0.0
        assert dirEntry.st_ctime > 0.0
        assert dirEntry.st_mtime > 0.0
        assert hasattr(dirEntry, "st_mode")
        contents[pathName] = dirEntry
    assert len(contents) == 187


def test_scandir_full(tempDir):
    sd = scandir.scandir.Scandir(tempDir.name,
                                 return_type=scandir.RETURN_TYPE_FULL)
    contents = {}
    for pathName, dirEntry in sd:
        assert dirEntry.st_atime > 0.0
        assert dirEntry.st_ctime > 0.0
        assert dirEntry.st_mtime > 0.0
        assert hasattr(dirEntry, "st_mode")
        contents[pathName] = dirEntry
    assert len(contents) == 187
