# Walk

The most simple way of using `Walk` is the example below. Use this if you just need the final results.

```rust
// collect() starts the worker thread and waits until it has finished. The line below is blocking.
let results = Walk::new(&path, None)?.collect()?;
```

If you need some more information, which `Walk` via `instance` provides then use the example below.

```rust
let mut instance = Walk::new(&path, None)?;
// collect() starts the worker thread and waits until it has finished. The line below is blocking.
let results = instance.collect()?;
```

The example below uses extended metadata to identify more file types. Of course, it is slower.

```rust
let mut instance = Walk::new(&path, None)?.return_type(ReturnType::Ext);
let results = instance.collect()?;
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
