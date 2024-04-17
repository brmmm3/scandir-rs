# -*- coding: utf-8 -*-

import os
import threading

from scandir_rs import Scandir, ReturnType


def Counter(sd):
    print("Counter started...")
    x = 0
    while not sd.finished:
        x += 1
    print(f"X={x}")


dirName = "C:/Windows/appcompat" if os.name == "nt" else "/tmp"

# ReturnType.Base is the default
print("\n*** return_type=ReturnType.Base:")
for dirEntry in Scandir(dirName, return_type=ReturnType.Base):
    print(dirEntry)

# Or collect all results and return them in one step
results = Scandir(dirName).collect()

# And with extended metadata
instance = Scandir(dirName)
# Use method `extended` as an alternative to the above usage.
instance.extended(True)
results, errors = instance.collect()
print("\n*** Scandir results_cnt=", len(results))
for dirEntry in results:
    print(dirEntry.path, dirEntry.ctime)

print("\n*** return_type=ReturnType.Ext:")
for dirEntry in Scandir(dirName, return_type=ReturnType.Ext):
    print(dirEntry)


print("*** Parallel Threads ***")
sd = Scandir(".", return_type=ReturnType.Ext)
thr = threading.Thread(target=Counter, args=(sd,), daemon=True)
thr.start()
sd.start()
thr.join()
results = sd.results()
print("Finished", sd.busy, sd.finished, sd.has_errors(), len(results))
print(str(results)[:200])
print(sd.statistics)
# Need to be compiled with feature "speedy"
# print(sd.to_speedy())
# Need to be compiled with feature "bincode"
# print(sd.to_bincode())
# Need to be compiled with feature "json"
# print(sd.to_json())
