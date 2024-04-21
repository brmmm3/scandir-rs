# Benchmarks

Benchmarking code see [benches/benchmark.py](../benches/benchmark.py)

In the below table the line **Walk.iter** returns comparable
results to os.walk.

## Linux with Tower Ryzen 5 2400G @ 3.6GHz (4/8 cores) and Samsung SSD 960 EVO 250GB (NVME, EXT4)

### Directory linux-5.9 with

- 4711 directories
- 69973 files
- 38 symlinks
- 1.08GB size and 1.23GB usage on disk

#### Count

|   Time [s] | Method                        |
|------------|-------------------------------|
|   0.016    | Count.collect                 |
|   0.026    | Count(Ext).collect |

#### Walk

|   Time [s] | Method                         |
|------------|--------------------------------|
|   0.149    | os.walk (Python 3.12.3)        |
|   0.044    | Walk.iter                      |
|   0.066    | Walk.collect                   |
|   0.531    | os.walk(Ext) (Python 3.12.3)   |
|   0.047    | Walk(Ext).iter                 |
|   0.065    | Walk(Ext).collect              |

Walk.iter **~3.3 times faster** than os.walk.  
Walk(Ext).iter **~12.1 times faster** than os.walk(Ext).

![](images/linux_walk_linux-5.9.png)

#### Scandir

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   0.442    | scantree (os.scandir, Python 3.12.3) |
|   0.067    | Scandir.iter                         |
|   0.084    | Scandir.collect                      |
|   0.101    | Scandir(Ext).iter                    |
|   0.122    | Scandir(Ext).collect                 |

Scandir.iter **~5.5 times faster** than scantree(os.scandir).  
Scandir(Ext).iter **~4.1 times faster** than scantree(os.scandir).

![](images/linux_scandir_linux-5.9.png)

### Directory /usr with

- 45061 directories
- 388526 files
- 34937 symlinks
- 177 hardlinks
- 0 devices
- 0 pipes
- 23.16GB size and 24.03GB usage on disk

#### Count

|   Time [s] | Method                        |
|------------|-------------------------------|
|   0.104    | Count.collect                 |
|   0.165    | Count(Ext).collect            |

#### Walk

|   Time [s] | Method                         |
|------------|--------------------------------|
|   1.340    | os.walk (Python 3.12.3)        |
|   0.271    | Walk.iter                      |
|   0.444    | Walk.collect                   |
|   3.773    | os.walk(Ext) (Python 3.12.3)   |
|   0.278    | Walk(Ext).iter                 |
|   0.439    | Walk(Ext).collect              |

Walk.iter **~4.9 times faster** than os.walk.  
Walk(Ext).iter **~13.0 times faster** than os.walk(Ext).

![](images/linux_walk_usr.png)

#### Scandir

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   2.785    | scantree (os.scandir, Python 3.12.3) |
|   0.430    | Scandir.iter                         |
|   0.668    | Scandir.collect                      |
|   0.596    | Scandir(Ext).iter                    |
|   0.874    | Scandir(Ext).collect                 |

Scandir.iter **~6.5 times faster** than scantree(os.scandir).  
Scandir(Ext).iter **~4.7 times faster** than scantree(os.scandir).

![](images/linux_scandir_usr.png)

## Windows 10 with Laptop Core i7-11850H @ 2.5GHz (8/16 cores) and Samsung MZVLB1T0HBLR-000H1 (NVME, NTFS)

### Directory linux-5.9 with

- 4712 directories
- 69998 files
- 1.08GB size and 1.23GB usage on disk

#### Count

|   Time [s] | Method             |
|------------|--------------------|
|  0.027     | Count.collect      |
|  0.276     | Count(Ext).collect |

#### Walk

|   Time [s] | Method                       |
|------------|------------------------------|
|  0.771     | os.walk (Python 3.12.3)      |
|  0.092     | Walk.iter                    |
|  0.128     | Walk.collect                 |
|  6.289     | os.walk(Ext) (Python 3.12.3) |
|  0.090     | Walk(Ext).iter               |
|  0.124     | Walk(Ext).collect            |

Walk.iter **~8.4 times faster** than os.walk.  
Walk(Ext).iter **~69.8 times faster** than os.walk(Ext).

![](images/windows_walk_linux-5.9.png)

#### Scandir

|   Time [s] | Method                               |
|------------|--------------------------------------|
|  0.611     | scantree (os.scandir, Python 3.12.3) |
|  0.094     | Scandir.iter                         |
|  0.132     | Scandir.collect                      |
|  0.860     | Scandir(Ext).iter                    |
|  0.892     | Scandir(Ext).collect                 |

Scandir.iter **~6.5 times faster** than scantree(os.scandir).  
Scandir(Ext).iter **slower** than scantree(os.scandir). **TODO:** Needs investigation why.

![](images/windows_scandir_linux-5.9.png)

### Directory C:\Windows with

- 212836 directories
- 428834 files
- 37428 hardlinks
- 42.77GB size and 43.91GB usage on disk

#### Count

|   Time [s] | Method             |
|------------|--------------------|
|    1.441   | Count.collect      |
|    4.670   | Count(Ext).collect |

#### Walk

|   Time [s] | Method                       |
|------------|------------------------------|
|   36.255   | os.walk (Python 3.12.3)      |
|    4.276   | Walk.iter                    |
|    5.366   | Walk.collect                 |
|   89.770   | os.walk(Ext) (Python 3.12.3) |
|    4.457   | Walk(Ext).iter               |
|    5.680   | Walk(Ext).collect            |

Walk.iter **~8.5 times faster** than os.walk.  
Walk(Ext).iter **~20.1 times faster** than os.walk(Ext).

![](images/windows_walk_windows.png)

#### Scandir

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   24.700   | scantree (os.scandir, Python 3.12.3) |
|    4.245   | Scandir.iter                         |
|    4.713   | Scandir.collect                      |
|   14.060   | Scandir(Ext).iter                    |
|   14.566   | Scandir(Ext).collect                 |

Scandir.iter **~5.8 times faster** than scantree(os.scandir).  
Scandir(Ext).iter **~1.8 times faster** than scantree(os.scandir).

![](images/windows_scandir_windows.png)
