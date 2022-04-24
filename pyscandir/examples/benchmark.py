#!/usr/bin/python3

import os
import sys
import time

from scandir_rs import Count, Walk, Scandir

if os.name == 'nt':
    dirName = "C:/Windows"
else:
    dirName = "~/workspace"
print(f"Benchmarking directory: {dirName}")
if os.name != 'nt':
    dirName = os.path.expanduser(dirName)
print(Count(dirName, extended=True).collect())
print()

t1 = time.time()
for root, dirs, files in os.walk(os.path.expanduser(dirName)):
    pass
dt = time.time() - t1
print(f"os.walk: {dt:.3f}")

t1 = time.time()
toc = Count(dirName).collect()
dt = time.time() - t1
print(f"scandir_rs.count.count: {dt:.3f}")

t1 = time.time()
toc = Count(dirName).collect()
dt = time.time() - t1
print(f"scandir_rs.count.Count: {dt:.3f}")

t1 = time.time()
for result in Walk(dirName):
    pass
dt = time.time() - t1
print(f"scandir_rs.walk.Walk: {dt:.3f}")

t1 = time.time()
toc = Walk(dirName).collect()
dt = time.time() - t1
print(f"scandir_rs.walk.toc: {dt:.3f}")

t1 = time.time()
W = Walk(dirName)
toc = W.collect()
dt = time.time() - t1
print(f"scandir_rs.walk.collect: {dt:.3f}, internal={W.duration()}")

t1 = time.time()
entries = Scandir(dirName)
dt = time.time() - t1
print(f"scandir_rs.scandir.entries: {dt:.3f}")

t1 = time.time()
entries = Scandir(dirName).collect()
dt = time.time() - t1
print(f"scandir_rs.scandir.Scandir.collect: {dt:.3f}")

t1 = time.time()
S = Scandir(dirName)
for entry in S:
    pass
dt = time.time() - t1
print(f"scandir_rs.scandir.Scandir.iter: {dt:.3f}")
