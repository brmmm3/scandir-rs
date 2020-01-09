`scandir-rs`
============

A fast directory scanner.

On Linux:

Benchmarking directory: ~/workspace
Statistics { dirs: 22845, files: 321354, slinks: 130, hlinks: 22849, devices: 4, pipes: 1, size: 4941126424, usage: 5799833600, errors: [], duration: 0.203 }

os.walk: 0.547
scandir_rs.count.count: 0.132
scandir_rs.count.Count: 0.142
scandir_rs.walk.Walk: 0.237
scandir_rs.walk.toc: 0.224
scandir_rs.walk.collect: 0.242, internal=0.206
scandir_rs.scandir.entries: 0.262
scandir_rs.scandir.entries(metadata=True): 0.344
scandir_rs.scandir.entries(metadata_ext=True): 0.336
scandir_rs.scandir.Scandir.collect: 0.280
scandir_rs.scandir.Scandir.iter: 0.262
scandir_rs.scandir.Scandir.iter(metadata_ext=True): 0.330
