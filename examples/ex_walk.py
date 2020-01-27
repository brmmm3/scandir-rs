#!/usr/bin/python3

import time
import sys

import scandir_rs as r


print("# return_type=TOC:")
for nr, result in enumerate(r.walk.Walk("~/workspace", return_type=r.RETURN_TYPE_BASE)):
    print(result)
    if nr > 3:
        break

print("\n# return_type=WALK:")
for nr, result in enumerate(r.walk.Walk("~/workspace")):
    print(result)
    if nr > 3:
        break

print("\n# return_type=WALKEXT:")
for nr, result in enumerate(r.walk.Walk("~/workspace", return_type=r.RETURN_TYPE_EXT)):
    print(result)
    if nr > 3:
        break
