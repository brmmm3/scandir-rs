# scandir

The Rust crate is called `scandir` and installable via `cargo`. On Linux it is **1.5 - 2.9 times faster** and on Windows **1.5 - 5.4 time faster** (see [benchmarks](doc/benchmarks.md)).

If you are just interested in directory statistics you can use the `Count`.

`scandir_rs` contains following classes:

- `Count` for determining statistics of a directory.
- `Walk` for getting names of directory entries.
- `Scandir` for getting detailed stats of directory entries.

For the API see:

- Class [Count](doc/count.md)
- Class [Walk](doc/walk.md)
- Class [Scandir](doc/scandir.md)

## Examples

`Collect` examples:

```rust
use scandir::Count;

// collect() starts the worker thread and waits until it has finished. The line below is blocking.
println!(Count::new("/usr")?.collect()?);
// Get extended statistics
println!(Count::new("/usr", return_type=ReturnType.Ext)?.collect()?);
```

The same, but asynchronously in background using a class instance:

```rust
use scandir::Count;

let mut instance = Count::new("/usr", return_type=ReturnType.Ext);
instance.start();  // Start scanning the directory
...
let values = instance.results();  // Returns the current statistics. Can be read at any time
...
if instance.busy() {  // Check if the task is still running.
...
instance.stop();  // If you want to cancel the task
...
instance.join();  // Wait for the instance to finish.
```

```rust
let mut instance = Count::new(&path)?;
instance.start()?;
loop {
    if !instance.busy() {
        break;
    }
    // Do something
    thread::sleep(Duration::from_millis(10));
}
// collect() immediately returns because the worker thread has already finished.
let statistics = instance.collect()?;
```

`Walk` example:

```rust
use scandir::Walk;

// Get basic file tree
println!(Walk::new("/usr")?.collect()?);
// Get file tree with extended file type identification. This is slower.
println!(Walk::new("/usr", return_type=ReturnType.Ext)?.collect()?);
```

If you want to have intermediate results, e.g. you want to show the progress to the user, the use the example below.

```rust
let mut instance = Walk::new(&path, None)?;
instance.start()?;
loop {
    if !instance.busy() {
        break;
    }
    let new_results = instance.results(true);
    // Do something
    thread::sleep(Duration::from_millis(10));
}
// collect() immediately returns because the worker thread has already finished.
let results = instance.collect()?;
```

`Scandir` example:

```rust
use scandir::Scandir;

// Get basic file metadata
println!(Scandir::new("/usr")?.collect()?);
// Get extended file metadata
println!(Scandir::new("/usr", return_type=ReturnType.Ext, None)?.collect()?);
```

If you want to have intermediate results, e.g. you want to show the progress to the user, the use the example below.

```rust
let mut instance = Scandir::new(&path, None)?;
instance.start()?;
loop {
    if !instance.busy() {
        break;
    }
    let new_results = instance.results(true);
    // Do something
    thread::sleep(Duration::from_millis(10));
}
// collect() immediately returns because the worker thread has already finished.
let results = instance.collect()?;
```
