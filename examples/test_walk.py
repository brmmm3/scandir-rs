#!/usr/bin/python3

import time
import sys

import scandir_rs as r

#print(r.walk.toc("~/workspace", sorted=True))

print("TOC:")
W = r.walk.Walk("~/workspace")

for result in W:
    print(result)

print("WALK:")

W = r.walk.Walk("~/workspace", iter_type=r.ITER_TYPE_WALK)

for result in W:
    print(result)

#print(W.list())
