# -*- coding: utf-8 -*-

import os
import sys
import json
import timeit
import platform
from typing import Dict

import psutil
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
    from diskinfo import DiskInfo
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
        import requests
        proxies = None
        userDnsDomain = os.environ.get("USERDNSDOMAIN")
        if userDnsDomain and userDnsDomain.endswith("SCH.COM"):
            proxies = {
                "http": "http://127.0.0.1:3129",
                "http": "https://127.0.0.1:3129",
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
        print("Extracting linux-5.9.tar.gz...")
        os.makedirs(LINUX_DIR)
        # cmdLine = f"tar xzf {LINUX_KERNEL_ARCHIVE} -C {LINUX_DIR}"
        cmdLine = f"7z x {LINUX_KERNEL_ARCHIVE} -so | 7z x -aoa -si -ttar -o{LINUX_DIR}"
        print(f"Running: {cmdLine}")
        os.system(cmdLine)
    return tempDir


def RunCountBenchmarks(dirName: str) -> Dict[str, float]:
    print(f"Running Count benchmarks in directory: {dirName}")
    print(scandir.Count(dirName).collect())
    stats = json.loads(scandir.Count(dirName, return_type=scandir.ReturnType.Ext).collect().to_json())
    print(stats)
    dtScandirCountCollect = timeit.timeit(
        f"""
scandir.Count('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Count (collect): {dtScandirCountCollect}")

    dtScandirCountCollectExt = timeit.timeit(
        f"""
scandir.Count('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Count(Ext) (collect): {dtScandirCountCollectExt}")
    return {
        "stats": stats,
        "dtScandirCountCollect": dtScandirCountCollect / 3,
        "dtScandirCountCollectExt": dtScandirCountCollectExt / 3}


def RunWalkBenchmarks(dirName: str) -> Dict[str, float]:
    print(f"Running Walk benchmarks in directory: {dirName}")
    dtOsWalk = timeit.timeit(
        f"""
for root, dirs, files in os.walk('{dirName}'):
    pass
    """,
        setup="import os",
        number=3,
    ) / 3
    print(f"os.walk {dtOsWalk}")

    dtOsWalkExt = timeit.timeit(
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
    ) / 3
    print(f"os.walk(Ext) {dtOsWalkExt}")

    dtScandirWalkIter = timeit.timeit(
        f"""
for result in scandir.Walk('{dirName}'):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Walk (iter): {dtScandirWalkIter}")

    dtScandirWalkIterExt = timeit.timeit(
        f"""
for result in scandir.Walk('{dirName}', return_type=scandir.ReturnType.Ext):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Walk(Ext) (iter): {dtScandirWalkIterExt}")

    dtScandirWalkCollect = timeit.timeit(
        f"""
toc = scandir.Walk('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Walk (collect): {dtScandirWalkCollect}")

    dtScandirWalkCollectExt = timeit.timeit(
        f"""
instance = scandir.Walk('{dirName}', return_type=scandir.ReturnType.Ext)
toc = instance.collect()
print(instance.duration)
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Walk(Ext) (collect): {dtScandirWalkCollectExt}")
    return {
        "dtOsWalk": dtOsWalk,
        "dtOsWalkExt": dtOsWalkExt,
        "dtScandirWalkIter": dtScandirWalkIter,
        "dtScandirWalkIterExt": dtScandirWalkIterExt,
        "dtScandirWalkCollect": dtScandirWalkCollect,
        "dtScandirWalkCollectExt": dtScandirWalkCollectExt}


def RunScandirBenchmarks(dirName: str) -> Dict[str, float]:
    print(f"Running Scandir benchmarks in directory: {dirName}")
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
    ) / 3
    print(f"scantree (os.scandir): {dtOsScandir}")

    dtScandirScandirIter = timeit.timeit(
        f"""
for result in scandir.Scandir('{dirName}'):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Scandir (iter): {dtScandirScandirIter}")

    dtScandirScandirIterExt = timeit.timeit(
        f"""
for result in scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Ext):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Scandir(Ext) (iter): {dtScandirScandirIterExt}")

    dtScandirScandirCollect = timeit.timeit(
        f"""
entries = scandir.Scandir('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Scandir (collect): {dtScandirScandirCollect}")

    dtScandirScandirCollectExt = timeit.timeit(
        f"""
entries = scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Scandir(Ext) (collect): {dtScandirScandirCollectExt}")
    return {
        "dtOsScandir": dtOsScandir,
        "dtScandirScandirIter": dtScandirScandirIter,
        "dtScandirScandirIterExt": dtScandirScandirIterExt,
        "dtScandirScandirCollect": dtScandirScandirCollect,
        "dtScandirScandirCollectExt": dtScandirScandirCollectExt}


def BenchmarkDir(path: str, bCount: bool, bWalk: bool, bScandir: bool):
    print()
    pyVersion = sys.version.split(" ")[0]
    stats = {}
    tableCount = []
    tableWalk = []
    tableScandir = []
    if bCount:
        stats.update(RunCountBenchmarks(path))
        tableCount = [
            [stats["dtScandirCountCollect"], "Count.collect"],
            [stats["dtScandirCountCollectExt"], "Count(Ext).collect"]]
    if bWalk:
        stats.update(RunWalkBenchmarks(path))
        tableWalk = [
            [stats["dtOsWalk"], f"os.walk (Python {pyVersion})"],
            [stats["dtScandirWalkIter"], "Walk.iter"],
            [stats["dtScandirWalkCollect"], "Walk.collect"],
            [stats["dtOsWalkExt"], f"os.walk(Ext) (Python {pyVersion})"],
            [stats["dtScandirWalkIterExt"], "Walk(Ext).iter"],
            [stats["dtScandirWalkCollectExt"], "Walk(Ext).collect"]]
    if bScandir:
        stats.update(RunScandirBenchmarks(path))
        tableScandir = [
            [stats["dtOsScandir"], f"scantree (os.scandir, Python {pyVersion})"],
            [stats["dtScandirScandirIter"], "Scandir.iter"],
            [stats["dtScandirScandirCollect"], "Scandir.collect"],
            [stats["dtScandirScandirIterExt"], "Scandir(Ext).iter"],
            [stats["dtScandirScandirCollectExt"], "Scandir(Ext).collect"]]
    uname = platform.uname()
    print(f"\n{uname.system} {uname.machine} (kernel={uname.release})")
    print("Physical cores:", psutil.cpu_count(logical=False))
    print("Total cores:", psutil.cpu_count(logical=True))
    cpufreq = psutil.cpu_freq()
    print(f"Max Frequency: {cpufreq.max:.2f}Mhz")
    if os.name == 'posix:':
        disk = GetDiskInfo()
        print(f"Disk: {disk[0]} ({disk[1]}, {disk[2]})")
    print()
    s = stats["stats"]
    print(f"Directory {path} with:")
    print(f"  {s['dirs']} directories")
    print(f"  {s['files']} files")
    print(f"  {s['slinks']} symlinks")
    print(f"  {s['hlinks']} hardlinks")
    print(f"  {s['devices']} devices")
    print(f"  {s['pipes']} pipes")
    print(f"  {s['size'] / GB:.2f}GB size and {s['usage'] / GB:.2f}GB usage on disk")
    print()
    if tableCount:
        print(tabulate(tableCount, headers=["Time [s]", "Method"], tablefmt="github"))
        print()
    if tableWalk:
        print(tabulate(tableWalk, headers=["Time [s]", "Method"], tablefmt="github"))
        print()
        print(f"Walk.iter **~{stats["dtOsWalk"] / stats["dtScandirWalkIter"]:.1f} times faster** than os.walk.")
        print(f"Walk(Ext).iter **~{stats["dtOsWalkExt"] / stats["dtScandirWalkIterExt"]:.1f} times faster** than os.walk(Ext).")
        print()
    if tableScandir:
        print(tabulate(tableScandir, headers=["Time [s]", "Method"], tablefmt="github"))
        print()
        print(
            f"Scandir.iter **~{stats["dtOsScandir"] / stats["dtScandirScandirIter"]:.1f} times faster** than scantree(os.scandir)."
        )
        print(
            f"Scandir(Ext).iter **~{stats["dtOsScandir"] / stats["dtScandirScandirIterExt"]:.1f} times faster** than scantree(os.scandir)."
        )
    with open(f"benchmark_results_{os.name}_{os.path.basename(path)}.json", "w") as F:
        F.write(json.dumps(stats))


if __name__ == "__main__":
    tempDir = CreateTestData()
    dirName = "C:/Windows" if os.name == "nt" else "/usr"
    if " --" in str(sys.argv):
        bCount = "--count" in sys.argv
        bWalk = "--walk" in sys.argv
        bScandir = "--scandir" in sys.argv
    else:
        bCount = True
        bWalk = True
        bScandir = True
    BenchmarkDir(tempDir, bCount, bWalk, bScandir)
    BenchmarkDir(dirName, bCount, bWalk, bScandir)
