"""
# Professional scandir-rs Benchmark — Comparing Three APIs

## API differences

  Count   Counts entries and returns Statistics (directories, files, symlinks, total size, etc.). Comparable to du -s.
  Walk    Yields (root, dirs, files) tuples, similar to os.walk.
  Scandir Returns DirEntry objects with metadata such as paths, sizes, timestamps, and permissions. Comparable to os.scandir plus stat.

## Return types

  Base (default) Basic metadata: path, size, timestamps, and file type.
  Ext            Extended metadata, including hard-link count, device number, and inode. Requires additional system calls.


## Usage

  Count(dir).collect()                               # Statistics
  Count(dir, return_type=ReturnType.Ext).collect()   # Statistics (includes hard-link count)

  Walk(dir)                                          # Iterate over (root, dirs, files)
  Walk(dir, return_type=ReturnType.Ext)              # Iterate over (root, dirs, files, symlinks, other, errors)
  Walk(dir).collect()                                # Toc (directory and file lists)

  Scandir(dir)                                       # Iterate over DirEntry objects
  Scandir(dir, return_type=ReturnType.Ext)           # Iterate over DirEntryExt (includes st_mode/atime/ctime/mtime)
  Scandir(dir).collect()                             # List[DirEntry]
  Scandir(dir, return_type=ReturnType.Ext).collect() # List[DirEntryExt]

## Run

  python benchmark.py                 # Run all tests
  python benchmark.py --count --walk  # Run only Count and Walk tests

## References

  examples/count.py, examples/walk.py, examples/scandir.py
"""

import json
import os
import platform
import subprocess
import sys
import timeit

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
    partition = next([
        p for p in psutil.disk_partitions(all=False) if p.mountpoint in ("/", "C:\\")
    ])
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
                "https": "http://127.0.0.1:3129",
            }
        r = requests.get(
            "https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.9.tar.gz",
            stream=True,
            proxies=proxies,
        )
        print("Downloading linux-5.9.tar.gz...")
        with open(LINUX_KERNEL_ARCHIVE, "wb") as F:
            F.writelines(r.iter_content(chunk_size=4096))
    if not os.path.exists(LINUX_DIR):
        print("Extracting linux-5.9.tar.gz...")
        os.makedirs(LINUX_DIR)
        # The path is a file-level constant, not user input; use subprocess to avoid a shell-injection vector.
        # Replace os.system("7z x ... | 7z x ...") with a Popen pipeline, explicit argv, and return-code checks.
        p1 = subprocess.Popen(["7z", "x", LINUX_KERNEL_ARCHIVE, "-so"], stdout=subprocess.PIPE)
        try:
            p2 = subprocess.Popen(
                ["7z", "x", "-aoa", "-si", "-ttar", f"-o{LINUX_DIR}"],
                stdin=p1.stdout,
            )
            p1.stdout.close()  # Allow p1 to receive SIGPIPE after p2 exits.
            rc2 = p2.wait()
        finally:
            rc1 = p1.wait()
        if rc1 != 0 or rc2 != 0:
            raise RuntimeError(f"extraction failed: 7z[1] rc={rc1}, 7z[2] rc={rc2}")
    return tempDir


def RunCountBenchmarks(dirName: str) -> dict[str, float]:
    """Count API — Counts entries without storing them, uses minimal memory, and returns Statistics"""
    print(f"Running Count benchmarks in directory: {dirName}")
    # Count(dir).collect() → Statistics { dirs, files, slinks, size, ... }
    print(scandir.Count(dirName).collect())
    # Count(dir, return_type=ReturnType.Ext).collect() → Statistics { ..., hlinks, devices, pipes }
    stats = json.loads(scandir.Count(dirName, return_type=scandir.ReturnType.Ext).collect().to_json())
    print(stats)

    # Count.collect (Base) — Basic statistics.
    dtScandirCountCollect = timeit.timeit(
        f"""
scandir.Count('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Count (collect): {dtScandirCountCollect}")

    # Count.collect (Ext) — extended statistics (including hard-link counts, etc.)
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
        "Count.collect(Ext)": dtScandirCountCollectExt / 3}


def RunWalkBenchmarks(dirName: str) -> dict[str, float]:
    """Walk API — similar to os.walk, returning (root, dirs, files) tuples"""
    print(f"Running Walk benchmarks in directory: {dirName}")

    # Python comparison: os.walk (without retrieving stat information)
    dtOsWalk = timeit.timeit(
        f"""
for root, dirs, files in os.walk('{dirName}'):
    pass
    """,
        setup="import os",
        number=3,
    ) / 3
    print(f"os.walk {dtOsWalk}")

    # Python comparison: os.walk + os.stat (simulating Ext mode)
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

    # Walk.iter (Base) — Walk(dir) → Iterates over (root, dirs, files) tuples.
    dtScandirWalkIter = timeit.timeit(
        f"""
for result in scandir.Walk('{dirName}'):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Walk (iter): {dtScandirWalkIter}")

    # Walk.iter (Ext) — Walk(dir, return_type=ReturnType.Ext) → Iterates over (root, dirs, files, symlinks, other, errors) tuples.
    dtScandirWalkIterExt = timeit.timeit(
        f"""
for result in scandir.Walk('{dirName}', return_type=scandir.ReturnType.Ext):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Walk(Ext) (iter): {dtScandirWalkIterExt}")

    # Walk.collect (Base) — Walk(dir).collect() → Toc { dirs: [...], files: [...] }
    dtScandirWalkCollect = timeit.timeit(
        f"""
toc = scandir.Walk('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Walk (collect): {dtScandirWalkCollect}")

    # Walk.collect (Ext) — Walk(dir, return_type=ReturnType.Ext).collect() → Toc (including extended information).
    dtScandirWalkCollectExt = timeit.timeit(
        f"""
toc = scandir.Walk('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Walk(Ext) (collect): {dtScandirWalkCollectExt}")
    return {
        "os.walk": dtOsWalk,
        "os.walk(Ext)": dtOsWalkExt,
        "Walk.iter": dtScandirWalkIter,
        "Walk.iter(Ext)": dtScandirWalkIterExt,
        "Walk.collect": dtScandirWalkCollect,
        "Walk.collect(Ext)": dtScandirWalkCollectExt}


def RunScandirBenchmarks(dirName: str) -> dict[str, float]:
    """Scandir API — Returns full metadata for each entry (DirEntry/DirEntryExt)."""
    print(f"Running Scandir benchmarks in directory: {dirName}")

    # Python comparison: recursive os.scandir + stat (the best native Python approach)
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

    # Scandir.iter (Base) — Scandir(dir) → iterate over DirEntry objects (path, size, timestamp, file type)
    dtScandirScandirIter = timeit.timeit(
        f"""
for result in scandir.Scandir('{dirName}'):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Scandir (iter): {dtScandirScandirIter}")

    # Scandir.iter (Ext) — Scandir(dir, return_type=ReturnType.Ext) → iterate over DirEntryExt objects
    dtScandirScandirIterExt = timeit.timeit(
        f"""
for result in scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Ext):
    pass
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Scandir(Ext) (iter): {dtScandirScandirIterExt}")

    # Scandir.collect (Base) — Scandir(dir).collect() → List[DirEntry] (collect all entries into memory)
    dtScandirScandirCollect = timeit.timeit(
        f"""
entries = scandir.Scandir('{dirName}').collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Scandir (collect): {dtScandirScandirCollect}")

    # Scandir.collect (Ext) — Scandir(dir, return_type=ReturnType.Ext).collect() → List[DirEntryExt]
    dtScandirScandirCollectExt = timeit.timeit(
        f"""
entries = scandir.Scandir('{dirName}', return_type=scandir.ReturnType.Ext).collect()
    """,
        setup="import scandir_rs as scandir",
        number=3,
    ) / 3
    print(f"scandir.Scandir(Ext) (collect): {dtScandirScandirCollectExt}")
    return {
        "scantree (os.scandir)": dtOsScandir,
        "Scandir.iter": dtScandirScandirIter,
        "Scandir.iter(Ext)": dtScandirScandirIterExt,
        "Scandir.collect": dtScandirScandirCollect,
        "Scandir.collect(Ext)": dtScandirScandirCollectExt}


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
            [stats["Count.collect(Ext)"], "Count(Ext).collect"]]
    if bWalk:
        stats.update(RunWalkBenchmarks(path))
        tableWalk = [
            [stats["os.walk"], f"os.walk (Python {pyVersion})"],
            [stats["Walk.iter"], "Walk.iter"],
            [stats["Walk.collect"], "Walk.collect"],
            [stats["os.walk(Ext)"], f"os.walk(Ext) (Python {pyVersion})"],
            [stats["Walk.iter(Ext)"], "Walk(Ext).iter"],
            [stats["Walk.collect(Ext)"], "Walk(Ext).collect"]]
    if bScandir:
        stats.update(RunScandirBenchmarks(path))
        tableScandir = [
            [stats["scantree (os.scandir)"], f"scantree (os.scandir, Python {pyVersion})"],
            [stats["Scandir.iter"], "Scandir.iter"],
            [stats["Scandir.collect"], "Scandir.collect"],
            [stats["Scandir.iter(Ext)"], "Scandir(Ext).iter"],
            [stats["Scandir.collect(Ext)"], "Scandir(Ext).collect"]]
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
        factor = stats['os.walk'] / stats['Walk.iter']
        print(f"Walk.iter **~{factor:.1f} times faster** than os.walk.")
        factor = stats['os.walk(Ext)'] / stats['Walk.iter(Ext)']
        print(f"Walk(Ext).iter **~{factor:.1f} times faster** than os.walk(Ext).")
        print()
    if tableScandir:
        print(tabulate(tableScandir, headers=["Time [s]", "Method"], tablefmt="github"))
        print()
        factor = stats["scantree (os.scandir)"] / stats["Scandir.iter"]
        print(f"Scandir.iter **~{factor:.1f} times faster** than scantree(os.scandir).")
        factor = stats["scantree (os.scandir)"] / stats["Scandir.iter(Ext)"]
        print(f"Scandir(Ext).iter **~{factor:.1f} times faster** than scantree(os.scandir).")
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
