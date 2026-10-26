# csharpscandir

Managed C# wrapper for the `cscandir` C ABI and `scandir` Rust crate. The wrapper
uses .NET P/Invoke and converts native results into managed records. Native
result buffers are always released through the ABI's matching free functions.

## Build and test

Run from the repository root:

```bash
make -C csharpscandir build
make -C csharpscandir test
```

The Makefile builds `cscandir` first. The native library is located from
`target/release` or `target/debug` automatically. Set `SCANDIR_NATIVE_LIBRARY`
to an explicit native library path to override discovery.

## Examples

One console application implements each of the five C examples:

| Command | C counterpart | Purpose |
| --- | --- | --- |
| `count` | `count.c` | Aggregate statistics, optionally with extended metadata |
| `scandir` | `scandir.c` | Sorted entry listing |
| `walk` | `walk.c` | Bounded collection with elapsed-time reporting |
| `cifs` | `cifs_ctime_check.c` | CIFS ctime check and Linux `statx` cross-check |
| `benchmark` | `benchmark.c` | Collect, collect-and-discard, count, and extended collect timings |

Examples run via `dotnet run --project csharpscandir/examples/examples.csproj
-- <command> ...` or the Makefile helpers:

```bash
make -C csharpscandir run-count ROOT=/tmp EXT=1
make -C csharpscandir run-scandir ROOT=/usr EXT=1
make -C csharpscandir run-walk ROOT=/usr
make -C csharpscandir run-cifs ROOT=/mnt/smb
make -C csharpscandir benchmark ROOT=/usr ITERATIONS=50
```

`EXT=1` requests extended metadata in count, scandir, and walk. The CIFS
cross-check uses Linux `statx`; on other platforms the scan still runs but the
cross-check is skipped. Like the C ABI, collect/count calls are synchronous and
do not expose the Rust examples' start/stop or progress-polling APIs.

## API

```csharp
using CSharpScandir;

var options = new ScandirOptions { Sorted = true, Metadata = MetadataKind.Extended };
ScanResult scan = Scandir.Collect("/tmp", options);
foreach (ScandirEntry entry in scan.Entries)
    Console.WriteLine(entry.Path);

ScandirStatistics stats = Scandir.Count("/tmp", options);
```

`Scandir.CollectDiscardingResults()` invokes the native collect call without
requesting entry or per-entry error outputs. This avoids wrapper materialization;
it still performs the native traversal and core result collection.

## Tests

`make -C csharpscandir test` runs the managed regression suite for invalid
arguments and glob patterns, basic/extended metadata, sorting, statistics,
filters, depth/file limits, discarded outputs, and repeated native allocations.