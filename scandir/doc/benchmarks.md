# Benchmarks

Benchmarking code see [benches/benchmark.rs](../benches/benchmark.rs)

**(Ext)** means reading extended metadata to be able to identify hardlinks and special file types, like pipes, devices.

## Linux with Tower Ryzen 5 2400G @ 3.6GHz (4/8 cores) and Samsung SSD 960 EVO 250GB (NVME, EXT4)

### Directory linux-5.9 with

- 4711 directories
- 69973 files
- 38 symlinks
- 1.08GB size and 1.23GB usage on disk

#### Count linux-5.9

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.046    | Count.collect                        |
|   0.081    | Count(Ext).collect                   |

#### Walk linux-5.9

|   Time [s]    | Method                            |
|---------------|-----------------------------------|
|   0.082       | walkdir.WalkDir                   |
|   0.462       | walkdir.WalkDir(Ext)              |
|   0.056       | Walk.collect                      |
|   0.055       | Walk(Ext).collect                 |

Walk.collect **~1.5 times faster** than walkdir.WalkDir.  
Walk(Ext).collect  **~8.4 times faster** than walkdir.WalkDir(Ext).  

![](images/linux_walk_linux-5.9.png)

#### Scandir linux-5.9

|   Time [s]    | Method                            |
|---------------|-----------------------------------|
|   0.199       | scan_dir.ScanDir                  |
|   0.383       | scan_dir.ScanDir(Ext)             |
|   0.073       | Scandir.collect                   |
|   0.116       | Scandir(Ext).collect              |

Scandir.collect **~2.7 times faster** than scan_dir.ScanDir.
Scandir(Ext).collect **~3.3 times faster** than scan_dir.ScanDir(Ext).

![](images/linux_scandir_linux-5.9.png)

### Directory /usr with

- 45060 directories
- 388518 files
- 34937 symlinks
- 177 hardlinks
- 0 devices
- 0 pipes
- 23.16GB size and 24.02GB usage on disk

#### Count /usr

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.306    | Count.collect                        |
|   0.515    | Count(Ext).collect                   |

#### Walk /usr

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.671    | walkdir.WalkDir                      |
|   2.829    | walkdir.WalkDir(Ext)                 |
|   0.405    | Walk.collect                         |
|   0.404    | Walk(Ext).collect                    |

Walk.collect **~1.7 times faster** than walkdir.WalkDir.  
Walk(Ext).collect  **~7.0 times faster** than walkdir.WalkDir(Ext).

![](images/linux_walk_usr.png)

#### Scandir /usr

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   1.474    | scan_dir.ScanDir                     |
|   2.575    | scan_dir.ScanDir(Ext)                |
|   0.615    | Scandir.collect                      |
|   0.822    | Scandir(Ext).collect                 |

Scandir.collect **~2.4 times faster** than scan_dir.ScanDir.  
Scandir(Ext).collect **~3.1 times faster** than scan_dir.ScanDir(Ext).

![](images/linux_scandir_usr.png)

## Windows 10 with Laptop Core i7-11850H @ 2.5GHz (8/16 cores) and Samsung MZVLB1T0HBLR-000H1 (NVME, NTFS)

### Directory linux-5.9 with

- 4712 directories
- 69998 files
- 1.08GB size and 1.23GB usage on disk

#### Count linux-5.9

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.688    | Count.collect                        |
|   0.0908   | Count(Ext).collect                   |

#### Walk linux-5.9

|   Time [s] | Method                               |
|---------------|-----------------------------------|
|   0.484       | walkdir.WalkDir                   |
|   0.090843    | walkdir.WalkDir(Ext)              |
|   0.1         | Walk.collect                      |
|   0.099       | Walk(Ext).collect                 |

Walk.collect **~4.8 times faster** than walkdir.WalkDir.  
Walk(Ext).collect  **~4.9 times faster** than walkdir.WalkDir.  

![](images/windows_walk_linux-5.9.png)

#### Scandir linux-5.9

|   Time [s] | Method                               |
|---------------|-----------------------------------|
|   0.436       | scan_dir.ScanDir                  |
|   0.20626     | scan_dir.ScanDir(Ext)             |
|   0.086       | Scandir.collect                   |
|   0.779       | Scandir(Ext).collect              |

Scandir.collect **~5.1 times faster** than scan_dir.ScanDir.
Scandir(Ext).collect **slower** than scan_dir.ScanDir.

![](images/windows_scandir_linux-5.9.png)

### Directory C:\Windows with

- 165926 directories
- 316866 files
- 35364 hardlinks
- 39.68GB size and 40.53GB usage on disk

#### Count C:\Windows

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.688    | Count.collect                        |
|   0.0908   | Count(Ext).collect                   |

#### Walk C:\Windows

|   Time [s] | Method                               |
|---------------|-----------------------------------|
|   15.257      | walkdir.WalkDir                   |
|   0.090843    | walkdir.WalkDir(Ext)              |
|   3.046       | Walk.collect                      |
|   2.961       | Walk(Ext).collect                 |

Walk.collect **~5.0 times faster** than walkdir.WalkDir.  
Walk(Ext).collect  **~5.2 times faster** than walkdir.WalkDir.  

![](images/windows_walk_windows.png)

#### Scandir C:\Windows

|   Time [s] | Method                               |
|---------------|-----------------------------------|
|   15.13       | scan_dir.ScanDir                  |
|   0.20626     | scan_dir.ScanDir(Ext)             |
|   2.784       | Scandir.collect                   |
|   10.162      | Scandir(Ext).collect              |

Scandir.collect **~5.4 times faster** than scan_dir.ScanDir.
Scandir(Ext).collect **~1.5 times faster** than scan_dir.ScanDir.

![](images/windows_scandir_windows.png)
