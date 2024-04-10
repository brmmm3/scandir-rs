# -*- coding: utf-8 -*-

import os
import sys
import timeit
import tarfile
import traceback
import platform
from typing import Dict

import requests
import psutil
from diskinfo import DiskInfo
from tabulate import tabulate

import scandir_rs as scandir

GB = 1024 * 1024 * 1024
if os.name == "nt":
    LINUX_DIR = "C:/Workspace/benches/linux-5.9"
    LINUX_KERNEL_ARCHIVE = "C:/Workspace/benches/linux-5.9.tar.gz"
else:
    LINUX_DIR = os.path.expanduser("~/Rust/_Data/benches/linux-5.9")
    LINUX_KERNEL_ARCHIVE = os.path.expanduser("~/Rust/_Data/benches/linux-5.9.tar.gz")


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


def CreateTestData():
    tempDir = os.path.dirname(LINUX_DIR)
    if not os.path.exists(tempDir):
        os.makedirs(tempDir)
    if not os.path.exists(LINUX_KERNEL_ARCHIVE):
        proxies = None
        userDnsDomain = os.environ.get("USERDNSDOMAIN")
        if userDnsDomain and userDnsDomain.ends_with("SCH.COM"):
            proxies = {
                "http": "http://127.0.0.1:3129",
                "https": "https://127.0.0.1:3129",
            }
        r = requests.get(
            "https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.9.tar.gz",
            stream=True,
            proxies=proxies,
        )
        print("Downloading linux-5.9.tar.gz...")
        with open(LINUX_KERNEL_ARCHIVE, "wb") as F:
            for chunk in r.iter_content(chunk_size=4096):
                F.write(chunk)
    if not os.path.exists(LINUX_DIR):
        abspath = os.path.abspath
        print("Extracting linux-5.9.tar.gz...")
        try:
            with tarfile.open(LINUX_KERNEL_ARCHIVE, "r:gz") as Z:
                destDir = os.path.dirname(LINUX_DIR)
                for member in Z.getmembers():
                    member_path = os.path.join(destDir, member.name)
                    if not abspath(member_path).startswith(abspath(destDir)):
                        raise Exception("Attempted Path Traversal in Tar File")
                Z.extractall(destDir)
        except:
            traceback.print_exc()
    return tempDir


def RunBenchmarks(dirName: str) -> Dict[str, float]:
    print(f"Benchmarking directory: {dirName}")
    print(scandir.Count(dirName).collect())
    stats = scandir.Count(dirName, return_type=scandir.ReturnType.Ext).collect()
    print(stats)

    # Count

    dtScandirCountCollect = timeit.timeit(
        f"""
scandir.Count('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Count (collect): {dtScandirCountCollect}")

    dtScandirCountCollectExt = timeit.timeit(
        f"""
scandir.Count('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Count(Ext) (collect): {dtScandirCountCollectExt}")

    # Walk

    dtOsWalk = timeit.timeit(
        f"""
for root, dirs, files in os.walk('{dirName}'):
    pass
    """,
        setup="import os",
        number=3,
    )
    print(f"os.walk {dtOsWalk}")

    dtOsWalkStat = timeit.timeit(
        f"""
dirStats = dict()
fileStats = dict()
for root, dirs, files in os.walk('{dirName}'):
    for dirName in dirs:
        pathName = root + '/' + dirName
        try:
            dirStats[pathName] = os.stat(pathName)
        except:
            pass
    for fileName in files:
        pathName = root + '/' + fileName
        try:
            fileStats[pathName] = os.stat(pathName)
        except:
            pass
    """,
        setup="import os",
        number=3,
    )
    print(f"os.walk (stat) {dtOsWalkStat}")

    dtScandirWalkIter = timeit.timeit(
        f"""
for result in scandir.Walk('{dirName}'):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Walk (iter): {dtScandirWalkIter}")

    dtScandirWalkIterExt = timeit.timeit(
        f"""
for result in scandir.Walk('{dirName}', return_type=scandir.ReturnType.Ext):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Walk(Ext) (iter): {dtScandirWalkIterExt}")

    dtScandirWalkCollect = timeit.timeit(
        f"""
toc = scandir.Walk('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Walk (collect): {dtScandirWalkCollect}")

    dtScandirWalkCollectExt = timeit.timeit(
        f"""
toc = scandir.Walk('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Walk(Ext) (collect): {dtScandirWalkCollectExt}")

    # Scandir

    dtOsScandir = timeit.timeit(
        f"""
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

dirs = 0
files = 0
symlinks = 0
size = 0
for entry in scantree(os.path.expanduser('{dirName}')):
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
    """,
        setup="import os",
        number=3,
    )
    print(f"scantree (os.scandir): {dtOsScandir}")

    dtScandirScandirIter = timeit.timeit(
        f"""
for result in scandir.Scandir('{dirName}'):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Scandir (iter): {dtScandirScandirIter}")

    dtScandirScandirIterExt = timeit.timeit(
        f"""
for result in scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Ext):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Scandir(Ext) (iter): {dtScandirScandirIterExt}")

    dtScandirScandirCollect = timeit.timeit(
        f"""
entries = scandir.Scandir('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Scandir (collect): {dtScandirScandirCollect}")

    dtScandirScandirCollectExt = timeit.timeit(
        f"""
entries = scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    )
    print(f"scandir.Scandir(Ext) (collect): {dtScandirScandirCollectExt}")
    return {
        "stats": stats,
        "dtScandirCountCollect": dtScandirCountCollect,
        "dtScandirCountCollectExt": dtScandirCountCollectExt,
        "dtOsWalk": dtOsWalk,
        "dtOsWalkStat": dtOsWalkStat,
        "dtScandirWalkIter": dtScandirWalkIter,
        "dtScandirWalkIterExt": dtScandirWalkIterExt,
        "dtScandirWalkCollect": dtScandirWalkCollect,
        "dtScandirWalkCollectExt": dtScandirWalkCollectExt,
        "dtOsScandir": dtOsScandir,
        "dtScandirScandirIter": dtScandirScandirIter,
        "dtScandirScandirIterExt": dtScandirScandirIterExt,
        "dtScandirScandirCollect": dtScandirScandirCollect,
        "dtScandirScandirCollectExt": dtScandirScandirCollectExt,
    }


def BenchmarkDir(path: str):
    stats = RunBenchmarks(path)
    pyVersion = sys.version.split(" ")[0]
    uname = platform.uname()
    print(f"\n{uname.system} {uname.machine} (kernel={uname.release})")
    print("Physical cores:", psutil.cpu_count(logical=False))
    print("Total cores:", psutil.cpu_count(logical=True))
    cpufreq = psutil.cpu_freq()
    print(f"Max Frequency: {cpufreq.max:.2f}Mhz")
    disk = GetDiskInfo()
    print(f"Disk: {disk[0]} ({disk[1]}, {disk[2]})")
    print()
    s = stats["stats"]
    print(f"Directory {path} with:")
    print(f"  {s.dirs} directories")
    print(f"  {s.files} files")
    print(f"  {s.slinks} symlinks")
    print(f"  {s.hlinks} hardlinks")
    print(f"  {s.devices} devices")
    print(f"  {s.pipes} pipes")
    print(f"  {s.size / GB:.2f}GB size and {s.usage / GB:.2f}GB usage on disk")
    print()
    table = [[stats["dtScandirCountCollect"], "Count.collect"],
             [stats["dtScandirCountCollectExt"], "Count(ReturnType=Ext).collect"],
             [stats["dtOsWalk"], f"os.walk (Python {pyVersion})"],
             [stats["dtOsWalkStat"], f"os.walk (stat) (Python {pyVersion})"],
             [stats["dtScandirWalkIter"], "Walk.iter"],
             [stats["dtScandirWalkIterExt"], "Walk(ReturnType=Ext).iter"],
             [stats["dtScandirWalkCollect"], "Walk.collect"],
             [stats["dtScandirWalkCollectExt"], "Walk(ReturnType=Ext).collect"],
             [stats["dtOsScandir"], f"scantree (os.scandir, Python {pyVersion})"],
             [stats["dtScandirScandirIter"], "Scandir.iter"],
             [stats["dtScandirScandirIterExt"], "Scandir(ReturnType=Ext).iter"],
             [stats["dtScandirScandirCollect"], "Scandir.collect"],
             [stats["dtScandirScandirCollectExt"], "Scandir(ReturnType=Ext).collect"]]
    print(tabulate(table, headers=["Time [s]", "Method"], tablefmt="github"))
    print()
    print(f"Walk.iter **~{stats["dtOsWalk"] / stats["dtScandirWalkIter"]:.1f} times faster** than os.walk.")
    print(f"Walk(Ext).iter **~{stats["dtOsWalkStat"] / stats["dtScandirWalkIterExt"]:.1f} times faster** than os.walk(stat).")
    print(
        f"Scandir.iter **~{stats["dtOsScandir"] / stats["dtScandirScandirIter"]:.1f} times faster** than scantree(os.scandir)."
    )


if __name__ == "__main__":
    tempDir = CreateTestData()
    dirName = "C:/Windows" if os.name == "nt" else "/usr"
    BenchmarkDir(tempDir)
    BenchmarkDir(dirName)
