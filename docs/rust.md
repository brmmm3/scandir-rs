---
title: Rust
---

The `scandir` crate provides native `Count`, `Walk`, and `Scandir` APIs. It
returns Rust-owned results and supports worker threads, progress/result polling,
cancellation, and (in current source) timeout-bounded collection.

## Add the crate

Add the current published `scandir` release to your application's
`Cargo.toml` (for example, `scandir = "2.10.1"` when that is the current
published version), then import the API you need:

```toml
[dependencies]
scandir = "2.10.1"
```

Use the version available from your package registry; unreleased repository
work may have APIs not present in an older published version.

## Count entries

```rust
use scandir::Count;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let statistics = Count::new("/var/log")?
        .extended(true)
        .collect()?;

    println!("files: {}, directories: {}", statistics.files, statistics.dirs);
    println!("logical size: {} bytes", statistics.size);
    println!("elapsed: {:.3}s", statistics.duration);
    Ok(())
}
```

`Count::collect()` starts the worker when needed and waits for the result. For
aggregate size and extended accounting, request extended metadata. Count does
not return one `DirEntry` per file.

## Collect entries

```rust
use scandir::{ReturnType, Scandir};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = Scandir::new("/var/log", None)?
        .sorted(true)
        .skip_hidden(true)
        .return_type(ReturnType::Ext)
        .collect()?;

    for item in &results.results {
        println!("{item:?}");
    }
    for (path, error) in &results.errors {
        eprintln!("{path}: {error}");
    }
    Ok(())
}
```

The first result may represent the root directory; descendant paths are
root-relative. `ReturnType::Base` is the default; `ReturnType::Ext` requests
additional platform metadata.

## Walk a tree

```rust
use scandir::Walk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tree = Walk::new("/var/log", None)?.max_depth(3).collect()?;
    println!("{tree:?}");
    Ok(())
}
```

For responsive applications, call `start()`, poll `busy()` and `results(true)`
for newly available results, and finish with `collect()`. Call `stop()` to
request cancellation. The unreleased 3.0.0 source also provides
`collect_timeout()`; a timeout returns `None` while the worker continues, so call
`stop()` if you want to abort it. This method is not part of the published
2.10.1 crate.

## Options

The fluent methods include `sorted`, `skip_hidden`, `max_depth`,
`max_file_cnt`, `dir_include`, `dir_exclude`, `file_include`, `file_exclude`,
`case_sensitive`, `follow_links`, and `extended` / `return_type`. A zero depth or
file-count limit means unlimited. See the [crate README](https://github.com/brmmm3/scandir-rs/blob/master/scandir/README.md)
and [API source](https://docs.rs/scandir/) for the exact API of your crate
version.
