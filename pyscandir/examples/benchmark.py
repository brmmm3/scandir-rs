#!/usr/bin/python3

import os
import sys
import time
import platform

import psutil
from diskinfo import DiskInfo
from tabulate import tabulate

from scandir_rs import Count, Walk, Scandir, ReturnType


def scantree(path):
    try:
        for entry in os.scandir(path):
            if entry.is_dir(follow_symlinks=False):
                yield entry
                yield from scantree(entry.path)
            else:
                yield entry
    except:
        return


def GetDiskInfo():
    partition = [
        p for p in psutil.disk_partitions(all=False) if p.mountpoint in ("/", "C:\\")
    ][0]
    disks = DiskInfo().get_disk_list()
    for disk in disks:
        if partition.device.startswith(disk.get_path()):
            return (
                disk.get_model(),
                ("SSD" if disk.is_ssd() else "NVME" if disk.is_nvme() else "HDD"),
                partition.fstype,
            )


dirName = "C:/Windows" if os.name == "nt" else "/usr"
pyVersion = sys.version.split(" ")[0]

print(f"Benchmarking directory: {dirName}")
if os.name != "nt":
    dirName = os.path.expanduser(dirName)
print(Count(dirName).collect())
print(Count(dirName, return_type=ReturnType.Ext).collect())
print()

table = []

t1 = time.time()
toc = Count(dirName).collect()
dt = time.time() - t1
print(f"Count.collect: {dt:.3f}")
table.append([f"{dt:.3}", "Count.collect"])

t1 = time.time()
toc = Count(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Count(ReturnType=Ext).collect: {dt:.3f}")
table.append([f"{dt:.3}", "Count(ReturnType=Ext).collect"])

t1 = time.time()
cnt = 0
for root, dirs, files in os.walk(os.path.expanduser(dirName)):
    cnt += 1
dtOsWalk = time.time() - t1
print(f"os.walk: {dtOsWalk:.3f} {cnt}")
table.append([f"{dtOsWalk:.3}", f"os.walk (Python {pyVersion})"])

t1 = time.time()
cnt = 0
for result in Walk(dirName):
    cnt += 1
dtWalkIter = time.time() - t1
print(f"Walk.iter: {dtWalkIter:.3f} {cnt}")
table.append([f"{dtWalkIter:.3}", "Walk.iter"])

t1 = time.time()
toc = Walk(dirName).collect()
dt = time.time() - t1
print(f"Walk.collect: {dt:.3f} dirs={len(toc.dirs)} files=={len(toc.files)}")
table.append([f"{dt:.3}", "Walk.collect"])

t1 = time.time()
instance = Walk(dirName)
toc = instance.collect()
dt = time.time() - t1
print(
    f"Walk.collect: {dt:.3f} dirs={len(toc.dirs)} files=={len(toc.files)} Walk().duration={instance.duration}"
)

t1 = time.time()
toc = Walk(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Walk(ReturnType=Ext).collect: {dt:.3f} {str(toc)[:500]}")
table.append([f"{dt:.3}", "Walk(ReturnType=Ext).collect"])

t1 = time.time()
dirs = 0
files = 0
symlinks = 0
size = 0
for entry in scantree(os.path.expanduser(dirName)):
    try:
        st = entry.stat()
    except:
        continue
    if entry.is_dir():
        dirs += 1
    elif entry.is_file():
        files += 1
    elif entry.is_symlink():
        symlinks += 1
    size += st.st_size
dtScantree = time.time() - t1
print(f"scantree (os.scandir): {dtScantree:.3f} {dirs=} {files=} {symlinks=} {size=}")
table.append([f"{dtScantree:.3}", f"scantree (os.scandir, Python {pyVersion})"])

t1 = time.time()
entries = Scandir(dirName).collect()
dt = time.time() - t1
print(f"Scandir.collect: {dt:.3f} {len(entries)}")
table.append([f"{dt:.3}", "Scandir.collect"])

t1 = time.time()
instance = Scandir(dirName)
cnt = 0
for entry in instance:
    cnt += 1
dtScandirIter = time.time() - t1
print(f"Scandir.iter: {dtScandirIter:.3f} {cnt}")
table.append([f"{dtScandirIter:.3}", "Scandir.iter"])

t1 = time.time()
instance = Scandir(dirName)
toc = instance.collect()
dt = time.time() - t1
print(f"Scandir.collect: {dt:.3f} {len(toc)} Scandir().duration={instance.duration}")

t1 = time.time()
entries = Scandir(dirName, return_type=ReturnType.Ext).collect()
dt = time.time() - t1
print(f"Scandir(ReturnType=Ext).collect: {dt:.3f} {len(entries)}")
table.append([f"{dt:.3}", "Scandir(ReturnType=Ext).collect"])

uname = platform.uname()
print(f"\n{uname.system} {uname.machine} (kernel={uname.release})")
print("Physical cores:", psutil.cpu_count(logical=False))
print("Total cores:", psutil.cpu_count(logical=True))
cpufreq = psutil.cpu_freq()
print(f"Max Frequency: {cpufreq.max:.2f}Mhz")
disk = GetDiskInfo()
print(f"Disk: {disk[0]} ({disk[1]}, {disk[2]})")
print()
print(tabulate(table, headers=["Time [s]", "Method"], tablefmt="github"))
print()
print(f"Walk.iter **~{dtOsWalk / dtWalkIter:.1f} times faster** than os.walk.")
print(
    f"Scandir.iter **~{dtScantree / dtScandirIter:.1f} times faster** than scantree(os.scandir)."
)
