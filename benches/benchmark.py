# -*- coding: utf-8 -*-

import os
import sys
import time
import tempfile
import timeit
import tarfile

import requests

import scandir_rs as scandir


def CreateTestData(tmpDirName=None):
    url = "https://mirrors.edge.kernel.org/pub/linux/kernel/v5.x/linux-5.5.5.tar.gz"
    r = requests.get(url, stream=True)
    tempDir = tempfile.TemporaryDirectory(prefix="scandir_rs_", dir=tmpDirName)
    tempZipPath = f"{tempDir.name}/linux-5.5.5.tar.gz"
    print("Downloading linux-5.5.5.tar.gz...")
    with open(tempZipPath, 'wb') as F:
        for chunk in r.iter_content(chunk_size=4096):
            F.write(chunk)
    print("Extracting linux-5.5.5.tar.gz...")
    with tarfile.open(tempZipPath, "r:gz") as Z:
        Z.extractall()
    return tempDir


def RunBenchmarks(dirName: str):
    print("scandir_rs.walk.Walk (iter): %.3f" % timeit.timeit(f"""
for result in scandir.walk.Walk('{dirName}', return_type=scandir.RETURN_TYPE_WALK):
    pass
    """, setup="import scandir_rs as scandir", number=3))

    print(f"os.walk: %.3f" % timeit.timeit(f"""
for root, dirs, files in os.walk('{dirName}'):
    pass
    """, setup="import os", number=3))

    print("os.walk (stat): %.3f" % timeit.timeit(f"""
dirStats = dict()
fileStats = dict()
for root, dirs, files in os.walk('{dirName}'):
    for dirName in dirs:
        pathName = root + '/' + dirName
        try:
            dirStats[pathName] = os.stat(pathName)
        except:
            pass
    for fileName in files:
        pathName = root + '/' + fileName
        try:
            fileStats[pathName] = os.stat(pathName)
        except:
            pass
    """, setup="import os", number=3))

    print("scandir_rs.count.count: %.3f" % timeit.timeit(f"""
scandir.count.count('{dirName}')
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.count.count(extended=True): %.3f" % timeit.timeit(f"""
scandir.count.count('{dirName}', extended=True)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.count.Count: %.3f" % timeit.timeit(f"""
toc = scandir.count.Count('{dirName}').collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.walk.toc: %.3f" % timeit.timeit(f"""
toc = scandir.walk.toc('{dirName}')
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.walk.Walk (iter): %.3f" % timeit.timeit(f"""
for result in scandir.walk.Walk('{dirName}', return_type=scandir.RETURN_TYPE_WALK):
    pass
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.walk.Walk (collect): %.3f" % timeit.timeit(f"""
toc = scandir.walk.Walk('{dirName}', return_type=scandir.RETURN_TYPE_WALK).collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (RETURN_TYPE_FAST): %.3f" % timeit.timeit(f"""
entries = scandir.scandir.entries('{dirName}', return_type=scandir.RETURN_TYPE_FAST)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (RETURN_TYPE_BASE): %.3f" % timeit.timeit(f"""
entries = scandir.scandir.entries('{dirName}', return_type=scandir.RETURN_TYPE_BASE)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (RETURN_TYPE_EXT): %.3f" % timeit.timeit(f"""
entries = scandir.scandir.entries('{dirName}', return_type=scandir.RETURN_TYPE_EXT)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (RETURN_TYPE_FULL): %.3f" % timeit.timeit(f"""
entries = scandir.scandir.entries('{dirName}', return_type=scandir.RETURN_TYPE_FULL)
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir_rs.scandir.entries (iter, RETURN_TYPE_FULL): %.3f" % timeit.timeit(f"""
for result in scandir.scandir.Scandir('{dirName}', return_type=scandir.RETURN_TYPE_FULL):
    pass
    """, setup="import scandir_rs as scandir", number=3))


if __name__ == "__main__":
    if os.name == 'nt':
        print(f"Benchmarking directory: C:/Windows")
        RunBenchmarks("C:/Windows")
    else:
        print(f"Benchmarking directory: /usr")
        RunBenchmarks("/usr")
    tempDir = CreateTestData(None if len(sys.argv) < 2 else sys.argv[1])
    try:
        print(f"Benchmarking directory: {tempDir.name}")
        RunBenchmarks(tempDir.name)
    finally:
        print("Cleanup...")
        tempDir.cleanup()
