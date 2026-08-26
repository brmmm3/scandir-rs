
import pytest
from scandir_rs import Count, ReturnType

from .common import CreateTempFileTree


@pytest.fixture(scope="session", autouse=True)
def tempDir():
    tmpDir = CreateTempFileTree(10, 3, 10)
    yield tmpDir
    tmpDir.cleanup()


def test_count(tempDir):
    count = Count(tempDir.name)
    count.start()
    count.join()
    assert count.as_dict() == {'dirs': 30, 'files': 900,
                               'size': 122880, 'usage': 122880}


def test_count_extended(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext).collect()
    assert count.as_dict() == {'dirs': 30, 'files': 900,
                               'size': 122880, 'usage': 122880}


def test_count_extended_file_exclude(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  file_exclude=["*.bin"]).collect()
    assert count.as_dict() == {'dirs': 30, 'files': 600,
                               'size': 122880, 'usage': 122880}


def test_count_extended_file_include(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  file_include=["*.bin"]).collect()
    assert count.as_dict() == {'dirs': 30, 'files': 300,
                               'size': 122880, 'usage': 122880}


def test_count_extended_dir_include(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  dir_include=["dir0/**"]).collect()
    assert count.as_dict() == {'dirs': 3, 'files': 90,
                               'size': 12288, 'usage': 12288}


def test_count_extended_dir_exclude(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  dir_exclude=["dir0", "dir1"]).collect()
    assert count.as_dict() == {'dirs': 24, 'files': 720,
                               'size': 98304, 'usage': 98304}