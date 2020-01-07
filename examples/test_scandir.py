#!/usr/bin/python3

import time
import sys

import scandir_rs as r

#print(r.walk.toc("~/workspace", sorted=True))

W = r.scandir.Walk("~/workspace")

for result in W:
    print(result)

#print(W.list())
