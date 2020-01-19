#!/usr/bin/python3

import time
import sys

import scandir_rs as r

"""
GC support currently NOT working!

import gc

def test():
    C = r.count.Count("~/workspace", extended=True)
    C.start()
    del C

test()

gc.collect()
sys.exit()
"""

C = r.count.Count("~/workspace", extended=True)

with C:
    while C.busy():
        print(C.statistics)
        time.sleep(0.01)
print("FINISHED")
print(C.statistics)
print(r.count.Count("~/workspace", extended=True).collect())
print(r.count.count("~/workspace", extended=True))

C = r.count.Count("~/workspace", extended=True)
C.start()
print(C.busy())
print(dir(C))
print(dir(C.statistics))
print(C.statistics)
time.sleep(0.5)
print(C.busy())
print(C.statistics)
print(C.statistics.dirs)
C.stop()
print(C.busy())
print(C.statistics)
print(C.statistics.duration)
print(C.has_results())
print(C.as_dict())
