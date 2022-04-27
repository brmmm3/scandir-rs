# -*- coding: utf-8 -*-

import os
import timeit
import tarfile
import traceback

import requests

import scandir_rs as scandir

if os.name == 'nt':
    LINUX_DIR = "C:/Workspace/benches/linux-5.9"
    LINUX_KERNEL_ARCHIVE = "C:/Workspace/benches/linux-5.9.tar.gz"
else:
    LINUX_DIR = "~/Rust/_Data/benches/linux-5.9"
    LINUX_KERNEL_ARCHIVE = "~/Rust/_Data/benches/linux-5.9.tar.gz"


def CreateTestData():
    tempDir = os.path.dirname(LINUX_DIR)
    if not os.path.exists(tempDir):
        os.makedirs(tempDir)
    if not os.path.exists(LINUX_KERNEL_ARCHIVE):
        proxies = None
        if os.environ.get("USERDNSDOMAIN").ends_with("BOSCH.COM"):
            proxies = {
                "http": "http://10.10.1.10:3128",
                "https": "https://10.10.1.10:1080",
            }
        r = requests.get("https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.9.tar.gz", stream=True, proxies=proxies)
        print("Downloading linux-5.9.tar.gz...")
        with open(LINUX_KERNEL_ARCHIVE, 'wb') as F:
            for chunk in r.iter_content(chunk_size=4096):
                F.write(chunk)
    if not os.path.exists(LINUX_DIR):
        print("Extracting linux-5.9.tar.gz...")
        try:
            with tarfile.open(LINUX_KERNEL_ARCHIVE, "r:gz") as Z:
                Z.extractall(os.path.dirname(LINUX_DIR))
        except:
            traceback.print_exc()
    return tempDir


def RunBenchmarks(dirName: str):
    print(f"Benchmarking directory: {dirName}")
    print(scandir.Count(dirName).collect())
    print(scandir.Count(dirName, return_type=scandir.ReturnType.Ext).collect())

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

    print("scandir.Count(): %.3f" % timeit.timeit(f"""
scandir.Count('{dirName}').collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir.Count(return_type=ReturnType.Ext).collect(): %.3f" % timeit.timeit(f"""
scandir.Count('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir.Walk().collect(): %.3f" % timeit.timeit(f"""
toc = scandir.Walk('{dirName}').collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir.Walk(return_type=scandir.ReturnType.Walk) (iter): %.3f" % timeit.timeit(f"""
for result in scandir.Walk('{dirName}', return_type=scandir.ReturnType.Walk):
    pass
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir.Walk(return_type=scandir.ReturnType.Walk) (collect): %.3f" % timeit.timeit(f"""
toc = scandir.Walk('{dirName}', return_type=scandir.ReturnType.Walk).collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir.Scandir(return_type=ReturnType.Fast).collect(): %.3f" % timeit.timeit(f"""
entries = scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Fast).collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir.Scandir(return_type=ReturnType.Base).collect(): %.3f" % timeit.timeit(f"""
entries = scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Base).collect()
    """, setup="import scandir_rs as scandir", number=3))

    print("scandir.Scandir(return_type=ReturnType.Ext).collect(): %.3f" % timeit.timeit(f"""
entries = scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """, setup="import scandir_rs as scandir", number=3))


if __name__ == "__main__":
    tempDir = CreateTestData()
    RunBenchmarks(tempDir)
