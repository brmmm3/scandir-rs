# Count

```rust
// collect() starts the worker thread and waits until it has finished. The line below is blocking.
let statistics = Count::new(&path)?.collect()?;
```

```rust
let mut instance = Count::new(&path)?;
// collect() starts the worker thread and waits until it has finished. The line below is blocking.
let statistics = instance.collect()?;
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
