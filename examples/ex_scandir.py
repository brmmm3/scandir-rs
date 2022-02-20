# -*- coding: utf-8 -*-

import os
import time
import threading

from scandir_rs import scandir, RETURN_TYPE_FAST, RETURN_TYPE_BASE, RETURN_TYPE_EXT, RETURN_TYPE_FULL


def Counter():
    print("Wait for busy...")
    # We need to use sub-module's thread safe ts_busy instead of busy method of Scandir instance!
    while not scandir.ts_busy():
        time.sleep(0.01)
    print("Counter started...")
    x = 0
    while scandir.ts_busy():
        x += 1
    print(f"X={x}")


def TsCounter():
    print("Wait for busy...")
    # We need to use sub-module's thread safe ts_busy instead of busy method of Scandir instance!
    while not scandir.ts_busy():
        time.sleep(0.01)
    print("Counter started...")
    x = 0
    while scandir.ts_busy():
        print(scandir.ts_count())
        time.sleep(0.5)


dirName = "C:/Windows/appcompat" if os.name == 'nt' else "/tmp"

print("*** return_type=RETURN_TYPE_FAST:")
for pathName, dirEntry in scandir.Scandir(dirName, return_type=RETURN_TYPE_FAST):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_BASE:")
for pathName, dirEntry in scandir.Scandir(dirName, return_type=RETURN_TYPE_BASE):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_EXT:")
for pathName, dirEntry in scandir.Scandir(dirName, return_type=RETURN_TYPE_EXT):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_FULL:")
S = scandir.Scandir(dirName, file_include=[
                    "*.txt"], return_type=RETURN_TYPE_FULL)
for pathName, dirEntry in S:
    print("#", S.busy(), pathName, dirEntry)

print("*** Parallel Threads ***")
thr = threading.Thread(target=Counter, daemon=True)
thr.start()
thr2 = threading.Thread(target=TsCounter, daemon=True)
thr2.start()
results = scandir.Scandir("C:/Windows", return_type=RETURN_TYPE_FULL).collect()
thr2.join()
thr.join()
print("Finished", S.busy(), S.has_results(), len(results))
