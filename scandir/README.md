# scandir

`scandir` scans directory trees in parallel. Use `Count` for statistics,
`Walk` for paths grouped by type, and `Scandir` for file metadata. See the
[benchmarks](https://github.com/brmmm3/scandir-rs/blob/master/scandir/doc/benchmarks.md)
and the API guides for
[Count](https://github.com/brmmm3/scandir-rs/blob/master/scandir/doc/count.md),
[Walk](https://github.com/brmmm3/scandir-rs/blob/master/scandir/doc/walk.md), and
[Scandir](https://github.com/brmmm3/scandir-rs/blob/master/scandir/doc/scandir.md).

## Count

```rust,no_run
use scandir::Count;

fn main() -> std::io::Result<()> {
    let statistics = Count::new(".")?.extended(true).collect()?;
    println!("{statistics:?}");
    Ok(())
}
```

## Walk

```rust,no_run
use scandir::Walk;

fn main() -> std::io::Result<()> {
    let paths = Walk::new(".", None)?.collect()?;
    println!("{:?}", paths.files);
    Ok(())
}
```

## Scandir

```rust,no_run
use scandir::Scandir;

fn main() -> std::io::Result<()> {
    let entries = Scandir::new(".", None)?.extended(true).collect()?;
    for entry in &entries.results {
        println!("{entry:?}");
    }
    Ok(())
}
```

For incremental results, start a worker and retrieve results while it runs:

```rust,no_run
use scandir::Scandir;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut scanner = Scandir::new(".", None)?;
    scanner.start()?;
    while scanner.busy() {
        let results = scanner.results_timeout(Duration::from_millis(100));
        println!("{} entries", results.results.len());
    }
    let remaining = scanner.results(true);
    println!("{} remaining entries", remaining.results.len());
    scanner.join();
    Ok(())
}
```

`max_file_cnt` limits non-directory entries, including symbolic links and
special files; `0` disables the limit. `Count` and `Scandir` include hidden
entries by default; `Walk` skips them by default.

Paths in `DirEntry`, `Toc`, walk results and scan errors use `PathBuf`, so
non-Unicode names and roots are preserved. This changes the earlier `String`
result API. Glob patterns are Unicode strings: when an active filter cannot
match a non-Unicode name, that entry is skipped and an error is reported.
Speedy, bincode and JSON retain their string-based formats and return an error
when a path is not Unicode.
