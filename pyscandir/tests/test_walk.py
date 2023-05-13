# -*- coding: utf-8 -*-

import time

import pytest
from scandir_rs import Walk, ReturnType

from .common import CreateTempFileTree


@pytest.fixture(scope="session", autouse=True)
def tempDir():
    tmpDir = CreateTempFileTree(10, 3, 10)
    yield tmpDir
    tmpDir.cleanup()


def test_walk_toc(tempDir):
    sd = Walk(tempDir.name, return_type=ReturnType.Ext)
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
    sd = Walk(tempDir.name, return_type=ReturnType.Base)
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
