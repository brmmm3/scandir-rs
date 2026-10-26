
import itertools

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
                  dir_include=["**/dir1/**"]).collect()
    assert count.as_dict() == {'dirs': 3, 'files': 90,
                               'size': 12288, 'usage': 12288}


def test_count_extended_dir_exclude(tempDir):
    count = Count(tempDir.name, return_type=ReturnType.Ext,
                  dir_exclude=["dir0/**", "dir1/**"]).collect()
    assert count.as_dict() == {'dirs': 24, 'files': 720,
                               'size': 98304, 'usage': 98304}


def test_count_iteration_yields_only_changed_statistics(tempDir):
    snapshots = [statistics.as_dict() for statistics in Count(tempDir.name)]

    assert snapshots
    assert all(previous != current for previous, current in itertools.pairwise(snapshots))
    assert snapshots[-1]["files"] == 900


def test_count_context_manager_does_not_suppress_value_error(tempDir):
    with pytest.raises(ValueError, match="caller error"), Count(tempDir.name):
        raise ValueError("caller error")


def test_count_iter_returns_active_iterator_unchanged(tempDir):
    iterator = iter(Count(tempDir.name))

    assert iter(iterator) is iterator
    snapshots = list(iterator)
    assert snapshots[-1].files == 900


@pytest.mark.parametrize("timeout", [-1.0, float("nan"), float("inf"), 1e19, 1e300])
def test_count_rejects_invalid_collect_timeout(tempDir, timeout):
    with pytest.raises(ValueError, match="timeout"):
        Count(tempDir.name).collect(timeout=timeout)


def test_count_collect_timeout_returns_statistics(tempDir):
    statistics = Count(tempDir.name).collect(timeout=10.0)

    assert statistics is not None
    assert statistics.files == 900


def test_count_root_relative_directory_include(tempDir):
    statistics = Count(
        tempDir.name,
        dir_include=["dir0/**"],
    ).collect()

    assert statistics.as_dict()["dirs"] == 3
    assert statistics.as_dict()["files"] == 90