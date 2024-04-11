# Scandir

```rust
// collect() starts the worker thread and waits until it has finished. The line below is blocking.
let results = Scandir::new(&path, None)?.collect()?;
```

```rust
let mut instance = Scandir::new(&path, None)?;
// collect() starts the worker thread and waits until it has finished. The line below is blocking.
let results = instance.collect()?;
```

```rust
let mut instance = Scandir::new(&path, None)?.return_type(ReturnType::Ext);
let results = instance.collect()?;
```

```rust
let mut instance = Scandir::new(&path, None)?;
instance.start()?;
loop {
    if !instance.busy() {
        break;
    }
    // Do something
    thread::sleep(Duration::from_millis(10));
}
// collect() immediately returns because the worker thread has already finished.
let results = instance.collect()?;
```
