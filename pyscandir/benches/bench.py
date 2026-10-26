"""
scandir-rs benchmark — Comparing Three APIs
=====================================

API differences:
  Count   → Counts entries and returns Statistics (dirs/files/slinks/size, etc.)  Similar to: du -s
  Walk    → Returns (root, dirs, files) tuples, similar to os.walk                Similar to: os.walk
  Scandir → Returns DirEntry objects with full metadata (path, size, timestamps, permissions, etc.)
                                                                                   Similar to: os.scandir + stat

Return types:
  Base (default) → Basic metadata (path, size, timestamps, file type)
  Ext            → Extended metadata (including hard-link count, device number, inode, etc.); requires extra system calls

Usage:
  Count(dir).collect()                           → Statistics
  Count(dir, return_type=ReturnType.Ext).collect() → Statistics (includes hlinks)

  Walk(dir)                                      → Iterate over (root, dirs, files)
  Walk(dir, return_type=ReturnType.Ext)          → Iterate over (root, dirs, files, symlinks, other, errors)
  Walk(dir).collect()                            → Toc (dirs + files lists)

  Scandir(dir)                                   → Iterate over DirEntry objects
  Scandir(dir, return_type=ReturnType.Ext)       → Iterate over DirEntryExt (includes st_mode/atime/ctime/mtime)
  Scandir(dir).collect()                         → List[DirEntry]
  Scandir(dir, return_type=ReturnType.Ext).collect() → List[DirEntryExt]

References: examples/count.py, examples/walk.py, examples/scandir.py
"""
import os
import time
from scandir_rs import Count, Walk, Scandir, ReturnType

target = 'Z:\\品质部\\内部文件\\MSDS模板\\2022'

# Warm up the filesystem cache
Count(target).collect()

results = []

# ========== Count — Counts only; stores no entries and uses minimal memory ==========
# Count(dir).collect() → Statistics { dirs, files, slinks, size, ... }
t1 = time.time()
s = Count(target).collect()
results.append(('Count.collect', time.time() - t1, s.dirs, s.files, s.size / 1e9))

# Count(dir, return_type=ReturnType.Ext).collect() → Statistics { dirs, files, hlinks, ... }
t1 = time.time()
s = Count(target, return_type=ReturnType.Ext).collect()
results.append(('Count(Ext).collect', time.time() - t1, s.dirs, s.files, s.hlinks))

# ========== Python standard-library comparisons ==========

# os.walk — Python's built-in recursive traversal (does not fetch stat data)
t1 = time.time()
dc = fc = 0
for r, d, f in os.walk(target):
    dc += len(d)
    fc += len(f)
oswalk_time = time.time() - t1
results.append(('os.walk', oswalk_time, dc, fc, 0))

# os.walk + os.stat — Python's built-in traversal with per-entry stat (simulates Ext mode)
t1 = time.time()
dc = fc = 0
for r, d, f in os.walk(target):
    for dn in d:
        try:
            os.stat(os.path.join(r, dn))
            dc += 1
        except OSError:
            pass
    for fn in f:
        try:
            os.stat(os.path.join(r, fn))
            fc += 1
        except OSError:
            pass
results.append(('os.walk(Ext)', time.time() - t1, dc, fc, 0))

# Recursive os.scandir — generator + stat (the best native Python approach)
t1 = time.time()
dc = fc = 0


def scantree(path):
    try:
        for entry in os.scandir(path):
            if entry.is_dir(follow_symlinks=False):
                yield entry
                yield from scantree(entry.path)
            else:
                yield entry
    except (PermissionError, OSError):
        return


for entry in scantree(target):
    try:
        entry.stat()
        if entry.is_dir():
            dc += 1
        else:
            fc += 1
    except OSError:
        pass
results.append(('scantree(os.scandir)', time.time() - t1, dc, fc, 0))

# ========== Walk — Similar to os.walk; returns (root, dirs, files) tuples ==========

# Walk(dir) → Iterate over (root, dirs, files)
t1 = time.time()
dc = fc = 0
for root, dirs, files in Walk(target):
    dc += len(dirs)
    fc += len(files)
results.append(('Walk.iter', time.time() - t1, dc, fc, 0))

# Walk(dir, return_type=ReturnType.Ext) → Iterate over (root, dirs, files, symlinks, other, errors)
t1 = time.time()
dc = fc = 0
for root, dirs, files, symlinks, other, errors in Walk(target, return_type=ReturnType.Ext):
    dc += len(dirs)
    fc += len(files)
results.append(('Walk(Ext).iter', time.time() - t1, dc, fc, len(symlinks)))

# Walk(dir).collect() → Toc { dirs: [...], files: [...] }
t1 = time.time()
toc = Walk(target).collect()
results.append(('Walk.collect', time.time() - t1, len(toc.dirs), len(toc.files), 0))

# Walk(dir, return_type=ReturnType.Ext).collect() → Toc (includes extended metadata)
t1 = time.time()
toc = Walk(target, return_type=ReturnType.Ext).collect()
results.append(('Walk(Ext).collect', time.time() - t1, len(toc.dirs), len(toc.files), 0))

# ========== Scandir — Returns full metadata for each entry (DirEntry/DirEntryExt) ==========

# Scandir(dir) → Iterate over DirEntry objects (path, size, timestamps, file type)
t1 = time.time()
cnt = 0
for entry in Scandir(target):
    cnt += 1
results.append(('Scandir.iter', time.time() - t1, cnt, 0, 0))

# Scandir(dir, return_type=ReturnType.Ext) → Iterate over DirEntryExt (also includes st_mode/atime/ctime/mtime)
t1 = time.time()
cnt = 0
for entry in Scandir(target, return_type=ReturnType.Ext):
    cnt += 1
results.append(('Scandir(Ext).iter', time.time() - t1, cnt, 0, 0))

# Scandir(dir).collect() → List[DirEntry] (collects all entries in memory)
t1 = time.time()
entries = Scandir(target).collect()
results.append(('Scandir.collect', time.time() - t1, len(entries), 0, 0))

# Scandir(dir, return_type=ReturnType.Ext).collect() → List[DirEntryExt]
t1 = time.time()
entries = Scandir(target, return_type=ReturnType.Ext).collect()
results.append(('Scandir(Ext).collect', time.time() - t1, len(entries), 0, 0))

# ========== Print results ==========
print()
print(f'=== scandir-rs benchmark (warm cache) ===')
print(f'| {"Method":<24}| {"Time":<8}| {"vs os.walk":<10}| {"Details":<20}|')
print(f'|{"-" * 24}|{"-" * 9}|{"-" * 11}|{"-" * 21}|')
for name, dt, *args in results:
    speedup = oswalk_time / dt
    detail = ' '.join(str(a) for a in args[:3])
    print(f'| {name:<23}| {dt:.3f}s  | {speedup:.1f}x       | {detail:<19}|')
print()
