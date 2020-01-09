#!/usr/bin/python3

import os
import sys
import time

import scandir_rs as r


t1 = time.time()
for root, dirs, files in os.walk(os.path.expanduser("~/workspace")):
    pass
dt = time.time() - t1
print(f"os.walk: {dt:.3f}")

t1 = time.time()
for result in r.walk.Walk("~/workspace", iter_type=r.ITER_TYPE_WALK):
    pass
dt = time.time() - t1
print(f"scandir_rs.walk.Walk: {dt:.3f}")

#print(W.list())
