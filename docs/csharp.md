---
title: C#
---

The `csharpscandir` module is a managed .NET wrapper over the `cscandir` C ABI.
It uses P/Invoke, copies returned entries into managed records, and releases
native buffers through their matching C ABI free functions. The library
project currently targets .NET 10.

## Build and test

From a repository checkout, build the wrapper (which builds `cscandir`) and
run its managed regression tests:

```sh
make -C csharpscandir build
make -C csharpscandir test
```

To use the source project from another .NET project, add a project reference:

```xml
<ItemGroup>
  <ProjectReference Include="../csharpscandir/csharpscandir.csproj" />
</ItemGroup>
```

Your application must also deploy the matching native `cscandir` library.
The wrapper searches the repository's `target/release` and `target/debug`
directories; set `SCANDIR_NATIVE_LIBRARY` to an explicit library path for a
different installation layout.

## Collect or count

```csharp
using CSharpScandir;

var options = new ScandirOptions
{
    Sorted = true,
    SkipHidden = true,
    Metadata = MetadataKind.Extended,
    FileInclude = ["*.log"],
};

ScanResult result = Scandir.Collect("/var/log", options);
foreach (ScandirEntry entry in result.Entries)
{
    Console.WriteLine(entry.Path);
}
foreach (string warning in result.Errors)
{
    Console.Error.WriteLine(warning);
}

ScandirStatistics stats = Scandir.Count("/var/log", options);
Console.WriteLine($"files={stats.Files}, bytes={stats.Size}");
```

`Scandir.Count()` returns aggregate values without materializing all entry
records. `Scandir.CollectDiscardingResults()` runs a native collect without
marshalling the entry or warning lists, but still performs the traversal and
native result collection. The `walk` example is built from collected results
and is not streaming. Fatal native failures throw `ScandirException`; child
traversal errors are returned separately. See the [C# README](https://github.com/brmmm3/scandir-rs/blob/master/csharpscandir/README.md)
for the full example and test commands.
