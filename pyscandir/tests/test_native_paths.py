import os

import pytest

from scandir_rs import Count, Scandir, Walk


@pytest.mark.skipif(os.name != "posix", reason="requires non-UTF-8 POSIX names")
def test_native_paths_round_trip(tmp_path):
    root = os.fsencode(tmp_path) + b"/root-\xff"
    nested = b"dir-\xfe"
    name = b"file-\xff"
    os.mkdir(root)
    os.mkdir(root + b"/" + nested)
    with open(root + b"/" + name, "wb") as file:
        file.write(b"x")
    with open(root + b"/" + nested + b"/" + name, "wb") as file:
        file.write(b"x")

    assert Count(root).collect().files == 2
    assert Count(tmp_path / os.fsdecode(b"root-\xff")).collect().files == 2
    expected = {name, nested + b"/" + name}
    for timeout in (None, 10.0):
        scan = Scandir(root).collect(timeout=timeout)
        assert not scan.errors
        assert {os.fsencode(entry.path) for entry in scan.files} == expected
        assert all(isinstance(entry.path, str) for entry in scan.files)
        assert {os.fsencode(path) for path in scan.as_dict() if path in {e.path for e in scan.files}} == expected
        toc = Walk(root).collect(timeout=timeout)
        assert not toc.errors
        assert {os.fsencode(path) for path in toc.files} == expected
        assert {os.fsencode(path) for path in toc.as_dict()["files"]} == expected
        for serialize in (toc.to_speedy, toc.to_bincode, toc.to_json):
            with pytest.raises(Exception, match="Unicode|UTF-8"):
                serialize()

    assert {
        os.fsencode(os.path.join(directory, name))
        for directory, _, files in Walk(os.fsdecode(root))
        for name in files
    } == expected
    assert {os.fsencode(entry.path) for entry in Scandir(root) if entry.is_file} == expected
