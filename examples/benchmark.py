#!/usr/bin/python3

import os
import sys
import time

import scandir_rs as r

if os.name == 'nt':
    dirName = "C:/Windows"
else:
    dirName = "~/workspace"
print(f"Benchmarking directory: {dirName}")
if os.name != 'nt':
    dirName = os.path.expanduser(dirName)
print(r.count.count(dirName, extended=True))
print()

t1 = time.time()
for root, dirs, files in os.walk(os.path.expanduser(dirName)):
    pass
dt = time.time() - t1
print(f"os.walk: {dt:.3f}")

t1 = time.time()
toc = r.count.count(dirName)
dt = time.time() - t1
print(f"scandir_rs.count.count: {dt:.3f}")

t1 = time.time()
toc = r.count.Count(dirName).collect()
dt = time.time() - t1
print(f"scandir_rs.count.Count: {dt:.3f}")

t1 = time.time()
for result in r.walk.Walk(dirName, iter_type=r.ITER_TYPE_WALK):
    pass
dt = time.time() - t1
print(f"scandir_rs.walk.Walk: {dt:.3f}")

t1 = time.time()
toc = r.walk.toc(dirName)
dt = time.time() - t1
print(f"scandir_rs.walk.toc: {dt:.3f}")

t1 = time.time()
W = r.walk.Walk(dirName)
toc = W.collect()
dt = time.time() - t1
print(f"scandir_rs.walk.collect: {dt:.3f}, internal={W.duration}")

t1 = time.time()
entries = r.scandir.entries(dirName)
dt = time.time() - t1
print(f"scandir_rs.scandir.entries: {dt:.3f}")

t1 = time.time()
entries = r.scandir.entries(dirName, metadata=True)
dt = time.time() - t1
print(f"scandir_rs.scandir.entries(metadata=True): {dt:.3f}")

t1 = time.time()
entries = r.scandir.entries(dirName, metadata_ext=True)
dt = time.time() - t1
print(f"scandir_rs.scandir.entries(metadata_ext=True): {dt:.3f}")

t1 = time.time()
entries = r.scandir.Scandir(dirName).collect()
dt = time.time() - t1
print(f"scandir_rs.scandir.Scandir.collect: {dt:.3f}")

t1 = time.time()
S = r.scandir.Scandir(dirName)
for entry in S:
    pass
dt = time.time() - t1
print(f"scandir_rs.scandir.Scandir.iter: {dt:.3f}")

t1 = time.time()
S = r.scandir.Scandir(dirName, metadata_ext=True)
for entry in S:
    pass
dt = time.time() - t1
print(f"scandir_rs.scandir.Scandir.iter(metadata_ext=True): {dt:.3f}")
