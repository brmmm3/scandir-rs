# Benchmarks

Benchmarking code see [benches/benchmark.rs](../benches/benchmark.rs)

## Linux with Tower Ryzen 5 2400G @ 3.6GHz (4/8 cores) and Samsung SSD 960 EVO 250GB (NVME, EXT4)

### Directory /usr with

- 45060 directories
- 388518 files
- 34937 symlinks
- 177 hardlinks
- 0 devices
- 0 pipes
- 23.16GB size and 24.02GB usage on disk

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.688    | walkdir.WalkDir                      |
|   0.431    | Walk.collect                         |
|   0.429    | Walk(ReturnType=Ext).collect         |
|   1.4842   | scan_dir.ScanDir                     |
|   0.63499  | Scandir.collect                      |
|   0.8931   | Scandir(ReturnType=Ext).collect      |

Walk.collect **~1.6 times faster** than walkdir.WalkDir.  
Walk(Ext).collect  **~1.6 times faster** than walkdir.WalkDir.  
Scandir.collect **~2.3 times faster** than scan_dir.ScanDir.
Scandir(Ext).collect **~1.7 times faster** than scan_dir.ScanDir.

#### Walk /usr

![](images/linux_walk_usr.png)

#### Scandir /usr

![](images/linux_scandir_usr.png)

### Directory linux-5.9 with

- 4711 directories
- 69973 files
- 38 symlinks
- 1.08GB size and 1.23GB usage on disk

|   Time [s]    | Method                            |
|---------------|-----------------------------------|
|   0.090843    | walkdir.WalkDir                   |
|   0.059257    | Walk.collect                      |
|   0.058337    | Walk(ReturnType=Ext).collect      |
|   0.20626     | scan_dir.ScanDir                  |
|   0.071707    | Scandir.collect                   |
|   0.11474     | Scandir(ReturnType=Ext).collect   |

Walk.collect **~1.5 times faster** than walkdir.WalkDir.  
Walk(Ext).collect  **~1.6 times faster** than walkdir.WalkDir.  
Scandir.collect **~2.9 times faster** than scan_dir.ScanDir.
Scandir(Ext).collect **~1.8 times faster** than scan_dir.ScanDir.

#### Walk linux-5.9

![](images/linux_walk_linux-5.9.png)

#### Scandir linux-5.9

![](images/linux_scandir_linux-5.9.png)

## Windows 10 with Laptop Core i7-11850H @ 2.5GHz (8/16 cores) and Samsung MZVLB1T0HBLR-000H1 (NVME, NTFS)

### Directory C:\Windows with

- 165926 directories
- 316866 files
- 35364 hardlinks
- 39.68GB size and 40.53GB usage on disk

|   Time [s] | Method                               |
|---------------|-----------------------------------|
|   15.257      | walkdir.WalkDir                   |
|   3.046       | Walk.collect                      |
|   2.961       | Walk(ReturnType=Ext).collect      |
|   15.13       | scan_dir.ScanDir                  |
|   2.784       | Scandir.collect                   |
|   10.162      | Scandir(ReturnType=Ext).collect   |

Walk.collect **~5.0 times faster** than walkdir.WalkDir.  
Walk(Ext).collect  **~5.2 times faster** than walkdir.WalkDir.  
Scandir.collect **~5.4 times faster** than scan_dir.ScanDir.
Scandir(Ext).collect **~1.5 times faster** than scan_dir.ScanDir.

#### Walk C:\Windows

![](images/windows_walk_windows.png)

#### Scandir C:\Windows

![](images/windows_scandir_windows.png)

### Directory linux-5.9 with

- 4712 directories
- 69998 files
- 1.08GB size and 1.23GB usage on disk

|   Time [s] | Method                               |
|---------------|-----------------------------------|
|   0.484       | walkdir.WalkDir                   |
|   0.1         | Walk.collect                      |
|   0.099       | Walk(ReturnType=Ext).collect      |
|   0.436       | scan_dir.ScanDir                  |
|   0.086       | Scandir.collect                   |
|   0.779       | Scandir(ReturnType=Ext).collect   |

Walk.collect **~4.8 times faster** than walkdir.WalkDir.  
Walk(Ext).collect  **~4.9 times faster** than walkdir.WalkDir.  
Scandir.collect **~5.1 times faster** than scan_dir.ScanDir.
Scandir(Ext).collect **slower** than scan_dir.ScanDir.

#### Walk linux-5.9

![](images/windows_walk_linux-5.9.png)

#### Scandir linux-5.9

![](images/windows_scandir_linux-5.9.png)
