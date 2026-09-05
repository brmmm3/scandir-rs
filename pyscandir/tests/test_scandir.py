
import pytest

from scandir_rs import ReturnType, Scandir

from .common import CreateTempFileTree


@pytest.fixture(scope="session", autouse=True)
def tempDir():
    tmpDir = CreateTempFileTree(10, 3, 10)
    yield tmpDir
    tmpDir.cleanup()


def test_scandir_fast(tempDir):
    sd = Scandir(tempDir.name, return_type=ReturnType.Base)
    contents = {}
    for dirEntry in sd:
        assert dirEntry.atime > 0.0
        assert dirEntry.ctime > 0.0
        assert dirEntry.mtime > 0.0
        assert not hasattr(dirEntry, "st_mode")
        contents[dirEntry.path] = dirEntry
    assert len(contents) == 930


def test_scandir_ext(tempDir):
    sd = Scandir(tempDir.name, return_type=ReturnType.Ext)
    contents = {}
    for dirEntry in sd:
        assert dirEntry.atime > 0.0
        assert dirEntry.ctime > 0.0
        assert dirEntry.mtime > 0.0
        assert hasattr(dirEntry, "st_mode")
        contents[dirEntry.path] = dirEntry
    assert len(contents) == 930
