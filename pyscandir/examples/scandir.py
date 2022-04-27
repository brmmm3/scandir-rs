# -*- coding: utf-8 -*-

import os
import time
import threading

from scandir_rs import Scandir, ReturnType


def Counter(sd):
    print("Counter started...")
    x = 0
    while not sd.finished():
        x += 1
    print(f"X={x}")


dirName = "C:/Windows/appcompat" if os.name == 'nt' else "/tmp"

print("*** return_type=ReturnType.Fast:")
for dirEntry in Scandir(dirName, return_type=ReturnType.Fast):
    print(dirEntry)


print("*** return_type=ReturnType.Base:")
for dirEntry in Scandir(dirName, return_type=ReturnType.Base):
    print(dirEntry)


print("*** return_type=ReturnType.Ext:")
for dirEntry in Scandir(dirName, return_type=ReturnType.Ext):
    print(dirEntry)


print("*** Parallel Threads ***")
sd = Scandir("~/workspace", return_type=ReturnType.Ext)
thr = threading.Thread(target=Counter, args=(sd,), daemon=True)
thr.start()
sd.start()
thr.join()
results = sd.results()
print("Finished", sd.busy(), sd.finished(), sd.has_errors(), len(results))
print(str(results)[:200])
