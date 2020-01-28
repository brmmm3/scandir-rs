# -*- coding: utf-8 -*-

import os
import sys
import time
import tempfile
import timeit

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


def RunBenchmarks(dirName: str):

    print(f"os.walk: %.3f" % timeit.timeit(f"""
for root, dirs, files in os.walk('{dirName}'):
    pass
    """, setup="import os", number=3))

    print(f"os.walk (stat): %.3f" % timeit.timeit(f"""
dirStats = dict()
fileStats = dict()
for root, dirs, files in os.walk('{dirName}'):
    for dirName in dirs:
        pathName = root + '/' + dirName
        dirStats[pathName] = os.stat(pathName)
    for fileName in files:
        pathName = root + '/' + fileName
        fileStats[pathName] = os.stat(pathName)
    """, setup="import os", number=3))

    print("scandir_rs.count.count:", timeit.timeit(f"""
scandir.count.count('{dirName}')
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.count.count(extended=True):", timeit.timeit(f"""
scandir.count.count('{dirName}', extended=True)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.count.Count:", timeit.timeit(f"""
toc = scandir.count.Count('{dirName}').collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.walk.toc:", timeit.timeit(f"""
toc = scandir.walk.toc('{dirName}')
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.walk.Walk (iter):", timeit.timeit(f"""
for result in scandir.walk.Walk('{dirName}', return_type=scandir.RETURN_TYPE_WALK):
    pass
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.walk.Walk (collect):", timeit.timeit(f"""
toc = scandir.walk.Walk('{dirName}', return_type=scandir.RETURN_TYPE_WALK).collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (RETURN_TYPE_FAST):", timeit.timeit(f"""
entries = scandir.scandir.entries('{dirName}', return_type=scandir.RETURN_TYPE_FAST)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (RETURN_TYPE_BASE):", timeit.timeit(f"""
entries = scandir.scandir.entries('{dirName}', return_type=scandir.RETURN_TYPE_BASE)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (RETURN_TYPE_EXT):", timeit.timeit(f"""
entries = scandir.scandir.entries('{dirName}', return_type=scandir.RETURN_TYPE_EXT)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (RETURN_TYPE_FULL):", timeit.timeit(f"""
entries = scandir.scandir.entries('{dirName}', return_type=scandir.RETURN_TYPE_FULL)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (iter, RETURN_TYPE_FULL):", timeit.timeit(f"""
for result in scandir.scandir.Scandir('{dirName}', return_type=scandir.RETURN_TYPE_FULL):
    pass
    """, setup="import scandir_rs as scandir", number=3))


if __name__ == "__main__":
    for dircnt, depth, filecnt in ((10, 3, 5000), (10, 10, 500)):
        tempDir = CreateTempFileTree(dircnt, depth, filecnt)
        print(f"Benchmarking directory: {tempDir.name}")
        RunBenchmarks(tempDir.name)
        print("Cleanup...")
        tempDir.cleanup()
        print()
    sys.exit(0)
