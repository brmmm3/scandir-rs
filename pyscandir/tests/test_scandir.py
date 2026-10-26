
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


def test_scandir_preserves_sorted_batch_order(tmp_path):
    expected = [f"entry-{index:03}.txt" for index in range(100)]
    for name in expected:
        (tmp_path / name).touch()

    scanner = Scandir(str(tmp_path), sorted=True)
    actual = [entry.path.rsplit("/", 1)[-1] for entry in scanner]

    assert actual == expected


def test_scandir_relative_directory_include_matches_root_child(tmp_path):
    (tmp_path / "included" / "nested").mkdir(parents=True)
    (tmp_path / "excluded").mkdir()
    (tmp_path / "included" / "nested" / "kept.txt").touch()
    (tmp_path / "excluded" / "omitted.txt").touch()

    scanner = Scandir(str(tmp_path), dir_include=["included/**"])
    paths = {
        entry.path.removeprefix(f"{tmp_path}/")
        for entry in scanner
    }

    assert paths == {"included", "included/nested", "included/nested/kept.txt"}


def test_scandir_context_manager_does_not_suppress_value_error(tmp_path):
    with pytest.raises(ValueError, match="caller error"), Scandir(str(tmp_path)):
        raise ValueError("caller error")


def test_scandir_iter_returns_active_iterator_unchanged(tmp_path):
    (tmp_path / "entry.txt").touch()
    iterator = iter(Scandir(str(tmp_path)))

    assert iter(iterator) is iterator
    assert [entry.path for entry in iterator] == ["entry.txt"]


@pytest.mark.parametrize("timeout", [-1.0, float("nan"), float("inf"), 1e19, 1e300])
def test_scandir_rejects_invalid_collect_timeout(tmp_path, timeout):
    with pytest.raises(ValueError, match="timeout"):
        Scandir(str(tmp_path)).collect(timeout=timeout)


@pytest.mark.parametrize("store", [False, True])
def test_scandir_collect_timeout_returns_results(tmp_path, store):
    (tmp_path / "entry.txt").touch()

    results = Scandir(str(tmp_path), store=store).collect(timeout=10.0)

    assert results is not None
    assert [entry.path.rsplit("/", 1)[-1] for entry in results.results] == ["entry.txt"]


@pytest.mark.parametrize("store", [False, True])
def test_scandir_collect_timeout_retry_preserves_results(tempDir, store):
    scanner = Scandir(tempDir.name, store=store)
    results = scanner.collect(timeout=1e-9)
    if results is None:
        results = scanner.collect(timeout=10.0)

    assert results is not None
    assert not results.errors
    paths = [entry.path for entry in results.results]
    assert len(paths) == len(set(paths)) == 930
