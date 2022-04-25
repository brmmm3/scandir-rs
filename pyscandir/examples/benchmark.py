#!/usr/bin/python3

import os
import sys
import time

from scandir_rs import Count, Walk, Scandir, ReturnType

if os.name == 'nt':
    dirName = "C:/Windows"
else:
    dirName = "/usr"
print(f"Benchmarking directory: {dirName}")
if os.name != 'nt':
    dirName = os.path.expanduser(dirName)
print(Count(dirName, return_type=ReturnType.Ext).collect())
print()

t1 = time.time()
for root, dirs, files in os.walk(os.path.expanduser(dirName)):
    pass
dt = time.time() - t1
print(f"os.walk: {dt:.3f}")

t1 = time.time()
toc = Count(dirName).collect()
dt = time.time() - t1
print(f"Count.collect: {dt:.3f}")

t1 = time.time()
toc = Count(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Count(ReturnType=Ext).collect: {dt:.3f}")

t1 = time.time()
for result in Walk(dirName):
    pass
dt = time.time() - t1
print(f"Walk.iter: {dt:.3f}")

t1 = time.time()
toc = Walk(dirName).collect()
dt = time.time() - t1
print(f"Walk.collect: {dt:.3f}")

t1 = time.time()
instance = Walk(dirName)
toc = instance.collect()
dt = time.time() - t1
print(f"Walk.collect: {dt:.3f}, Walk().duration={instance.duration()}")

t1 = time.time()
toc = Walk(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Walk(ReturnType=Ext).collect: {dt:.3f}")

t1 = time.time()
entries = Scandir(dirName).collect()
dt = time.time() - t1
print(f"Scandir.collect: {dt:.3f}")

t1 = time.time()
instance = Scandir(dirName)
for entry in instance:
    pass
dt = time.time() - t1
print(f"Scandir.iter: {dt:.3f}")

t1 = time.time()
instance = Scandir(dirName)
toc = instance.collect()
dt = time.time() - t1
print(f"Scandir.collect: {dt:.3f}, Scandir().duration={instance.duration()}")

t1 = time.time()
entries = Scandir(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Scandir(ReturnType=Ext).collect: {dt:.3f}")
