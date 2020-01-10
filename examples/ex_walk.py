#!/usr/bin/python3

import time
import sys

import scandir_rs as r


print("# iter_type=TOC:")
for nr, result in enumerate(r.walk.Walk("~/workspace", iter_type=r.ITER_TYPE_TOC)):
    print(result)
    if nr > 3:
        break

print("\n# iter_type=WALK:")
for nr, result in enumerate(r.walk.Walk("~/workspace")):
    print(result)
    if nr > 3:
        break

print("\n# iter_type=WALKEXT:")
for nr, result in enumerate(r.walk.Walk("~/workspace", iter_type=r.ITER_TYPE_WALKEXT)):
    print(result)
    if nr > 3:
        break
