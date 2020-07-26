# -*- coding: utf-8 -*-

import os
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


def test_count():
    tempDir = CreateTempFileTree(10, 3, 10)
    count = scandir.count.count(tempDir.name)
    tempDir.cleanup()
    assert count.as_dict() == {'dirs': 7, 'files': 180}


def test_count_extended():
    tempDir = CreateTempFileTree(10, 3, 10)
    count = scandir.count.count(tempDir.name, extended=True)
    tempDir.cleanup()
    assert count.as_dict() == {'dirs': 7, 'files': 180,
                               'size': 28672, 'usage': 28672}


def test_count_extended_file_exclude():
    tempDir = CreateTempFileTree(10, 3, 10)
    count = scandir.count.count(
        tempDir.name, extended=True, file_exclude=["*.bin"])
    tempDir.cleanup()
    assert count.as_dict() == {'dirs': 7, 'files': 120,
                               'size': 28672, 'usage': 28672}


def test_count_extended_file_include():
    tempDir = CreateTempFileTree(10, 3, 10)
    count = scandir.count.count(
        tempDir.name, extended=True, file_include=["*.bin"])
    tempDir.cleanup()
    assert count.as_dict() == {'dirs': 7, 'files': 60,
                               'size': 28672, 'usage': 28672}


def test_count_extended_dir_include():
    tempDir = CreateTempFileTree(10, 3, 10)
    count = scandir.count.count(
        tempDir.name, extended=True, dir_include=["dir0/**"])
    tempDir.cleanup()
    assert count.as_dict() == {'dirs': 4, 'files': 90,
                               'size': 16384, 'usage': 16384}


def test_count_extended_dir_exclude():
    tempDir = CreateTempFileTree(10, 3, 10)
    count = scandir.count.count(
        tempDir.name, extended=True, dir_exclude=["dir0", "dir1"])
    tempDir.cleanup()
    print(count.as_dict())
    assert count.as_dict() == {'dirs': 2, 'files': 30,
                               'size': 8192, 'usage': 8192}
