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
|   0.145235 | Count.collect                 |
|   0.246698 | Count(Ext).collect |

#### Walk

|   Time [s] | Method                         |
|------------|--------------------------------|
|   0.440817 | os.walk (Python 3.12.3)        |
|   0.133679 | Walk.iter                      |
|   0.197038 | Walk.collect                   |
|   1.61088  | os.walk(Ext) (Python 3.12.3)   |
|   0.133556 | Walk(Ext).iter                 |
|   0.191944 | Walk(Ext).collect              |

Walk.iter **~3.3 times faster** than os.walk.  
Walk(Ext).iter **~12.1 times faster** than os.walk(Ext).

![](images/linux_walk_linux-5.9.png)

#### Scandir

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   1.31862  | scantree (os.scandir, Python 3.12.3) |
|   0.237867 | Scandir.iter                         |
|   0.271947 | Scandir.collect                      |
|   0.320545 | Scandir(Ext).iter                    |
|   0.380465 | Scandir(Ext).collect                 |

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
|   0.880686 | Count.collect                 |
|   1.39832  | Count(Ext).collect            |

#### Walk

|   Time [s] | Method                         |
|------------|--------------------------------|
|   3.94502  | os.walk (Python 3.12.3)        |
|   0.80265  | Walk.iter                      |
|   1.34461  | Walk.collect                   |
|  10.7779   | os.walk(Ext) (Python 3.12.3)   |
|   0.827304 | Walk(Ext).iter                 |
|   1.33137  | Walk(Ext).collect              |

Walk.iter **~4.9 times faster** than os.walk.  
Walk(Ext).iter **~13.0 times faster** than os.walk(Ext).

![](images/linux_walk_usr.png)

#### Scandir

|   Time [s] | Method                               |
|------------|--------------------------------------|
|    8.25362 | scantree (os.scandir, Python 3.12.3) |
|    1.27802 | Scandir.iter                         |
|    2.01097 | Scandir.collect                      |
|    1.75471 | Scandir(Ext).iter                    |
|    2.58515 | Scandir(Ext).collect                 |

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
|  0.0270922 | Count.collect      |
|  0.275738  | Count(Ext).collect |

#### Walk

|   Time [s] | Method                       |
|------------|------------------------------|
|  0.771403  | os.walk (Python 3.12.3)      |
|  0.0919733 | Walk.iter                    |
|  0.12844   | Walk.collect                 |
|  6.28923   | os.walk(Ext) (Python 3.12.3) |
|  0.0901381 | Walk(Ext).iter               |
|  0.123835  | Walk(Ext).collect            |

Walk.iter **~8.4 times faster** than os.walk.  
Walk(Ext).iter **~69.8 times faster** than os.walk(Ext).

![](images/linux_walk_linux-5.9.png)

#### Scandir

|   Time [s] | Method                               |
|------------|--------------------------------------|
|  0.611043  | scantree (os.scandir, Python 3.12.3) |
|  0.0935449 | Scandir.iter                         |
|  0.132047  | Scandir.collect                      |
|  0.859825  | Scandir(Ext).iter                    |
|  0.892622  | Scandir(Ext).collect                 |

Scandir.iter **~6.5 times faster** than scantree(os.scandir).  
Scandir(Ext).iter **slower** than scantree(os.scandir). **TODO:** Needs investigation why.

![](images/linux_scandir_linux-5.9.png)

### Directory C:\Windows with

- 212836 directories
- 428834 files
- 37428 hardlinks
- 42.77GB size and 43.91GB usage on disk

#### Count

|   Time [s] | Method             |
|------------|--------------------|
|    1.44103 | Count.collect      |
|    4.67004 | Count(Ext).collect |

#### Walk

|   Time [s] | Method                       |
|------------|------------------------------|
|   36.2558  | os.walk (Python 3.12.3)      |
|    4.27589 | Walk.iter                    |
|    5.36591 | Walk.collect                 |
|   89.7699  | os.walk(Ext) (Python 3.12.3) |
|    4.45721 | Walk(Ext).iter               |
|    5.67997 | Walk(Ext).collect            |

Walk.iter **~8.5 times faster** than os.walk.  
Walk(Ext).iter **~20.1 times faster** than os.walk(Ext).

![](images/windows_walk_windows.png)

#### Scandir

|   Time [s] | Method                               |
|------------|--------------------------------------|
|   24.6999  | scantree (os.scandir, Python 3.12.3) |
|    4.24464 | Scandir.iter                         |
|    4.71281 | Scandir.collect                      |
|   14.0603  | Scandir(Ext).iter                    |
|   14.5655  | Scandir(Ext).collect                 |

Scandir.iter **~5.8 times faster** than scantree(os.scandir).  
Scandir(Ext).iter **~1.8 times faster** than scantree(os.scandir).

![](images/windows_scandir_windows.png)
