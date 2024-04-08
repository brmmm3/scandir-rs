#!/usr/bin/python3

import os
import time

from scandir_rs import Count, Walk, Scandir, ReturnType


def scantree(path):
    try:
        for entry in os.scandir(path):
            if entry.is_dir(follow_symlinks=False):
                yield entry
                yield from scantree(entry.path)
            else:
                yield entry
    except:
        return


if os.name == "nt":
    dirName = "C:/Windows"
else:
    dirName = "/usr"

print(f"Benchmarking directory: {dirName}")
if os.name != "nt":
    dirName = os.path.expanduser(dirName)
print(Count(dirName).collect())
print(Count(dirName, return_type=ReturnType.Ext).collect())
print()

t1 = time.time()
cnt = 0
for root, dirs, files in os.walk(os.path.expanduser(dirName)):
    cnt += 1
dt = time.time() - t1
print(f"os.walk: {dt:.3f} {cnt}")

t1 = time.time()
dirs = 0
files = 0
symlinks = 0
size = 0
for entry in scantree(os.path.expanduser(dirName)):
    try:
        st = entry.stat()
    except:
        continue
    if entry.is_dir():
        dirs += 1
    elif entry.is_file():
        files += 1
    elif entry.is_symlink():
        symlinks += 1
    size += st.st_size
dt = time.time() - t1
print(f"scantree: {dt:.3f} {dirs=} {files=} {symlinks=} {size=}")

t1 = time.time()
toc = Count(dirName).collect()
dt = time.time() - t1
print(f"Count.collect: {dt:.3f}")

t1 = time.time()
toc = Count(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Count(ReturnType=Ext).collect: {dt:.3f}")

t1 = time.time()
cnt = 0
for result in Walk(dirName):
    cnt += 1
dt = time.time() - t1
print(f"Walk.iter: {dt:.3f} {cnt}")

t1 = time.time()
toc = Walk(dirName).collect()
dt = time.time() - t1
print(f"Walk.collect: {dt:.3f} dirs={len(toc.dirs)} files=={len(toc.files)}")

t1 = time.time()
instance = Walk(dirName)
toc = instance.collect()
dt = time.time() - t1
print(
    f"Walk.collect: {dt:.3f} dirs={len(toc.dirs)} files=={len(toc.files)} Walk().duration={instance.duration()}"
)

t1 = time.time()
toc = Walk(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Walk(ReturnType=Ext).collect: {dt:.3f} {str(toc)[:500]}")

t1 = time.time()
entries = Scandir(dirName).collect()
dt = time.time() - t1
print(f"Scandir.collect: {dt:.3f} {len(entries)}")

t1 = time.time()
instance = Scandir(dirName)
cnt = 0
for entry in instance:
    cnt += 1
dt = time.time() - t1
print(f"Scandir.iter: {dt:.3f} {cnt}")

t1 = time.time()
instance = Scandir(dirName)
toc = instance.collect()
dt = time.time() - t1
print(f"Scandir.collect: {dt:.3f} {len(toc)} Scandir().duration={instance.duration()}")

t1 = time.time()
entries = Scandir(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Scandir(ReturnType=Ext).collect: {dt:.3f} {len(entries)}")
