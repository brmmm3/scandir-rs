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

Get statistics of a directory:

```rust
use scandir::Count;

// Get basic statistics
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

`Walk` example:

```rust
use scandir::Walk;

// Get basic statistics
println!(Walk::new("/usr")?.collect()?);
// Get extended statistics
println!(Walk::new("/usr", return_type=ReturnType.Ext)?.collect()?);
```

`Scandir` example:

```rust
use scandir::Scandir;

// Get basic statistics
println!(Scandir::new("/usr")?.collect()?);
// Get extended statistics
println!(Scandir::new("/usr", return_type=ReturnType.Ext, None)?.collect()?);
```
