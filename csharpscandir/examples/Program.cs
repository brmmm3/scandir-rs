using System.Diagnostics;
using System.Globalization;
using System.Runtime.InteropServices;
using CSharpScandir;

return Run(args);

static int Run(string[] args)
{
    CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
    CultureInfo.CurrentUICulture = CultureInfo.InvariantCulture;
    if (args.Length == 0)
    {
        Console.Error.WriteLine("Usage: examples <count|scandir|walk|cifs|benchmark> [arguments]");
        return 1;
    }

    try
    {
        return args[0] switch
        {
            "count" => Count(args[1..]),
            "scandir" => Scan(args[1..]),
            "walk" => Walk(args[1..]),
            "cifs" => Cifs(args[1..]),
            "benchmark" => Benchmark(args[1..]),
            _ => Usage(args[0]),
        };
    }
    catch (ScandirException error)
    {
        Console.Error.WriteLine($"scandir failed ({(int)error.Code}): {error.Message}");
        return 1;
    }
    catch (ArgumentException error)
    {
        Console.Error.WriteLine(error.Message);
        return 1;
    }
}

static int Usage(string command)
{
    Console.Error.WriteLine($"Unknown example '{command}'. Choose count, scandir, walk, cifs, or benchmark.");
    return 1;
}

static int Count(string[] args)
{
    if (!TryRootAndExtended(args, "/tmp", out string root, out bool extended))
    {
        Console.Error.WriteLine("Usage: examples count [ROOT] [--ext]");
        return 1;
    }

    var options = new ScandirOptions
    {
        DirectoryExclude = ["dir0", "dir1"],
        Metadata = extended ? MetadataKind.Extended : MetadataKind.Basic,
    };
    ScandirStatistics stats = Scandir.Count(root, options);
    Console.WriteLine($"root={root} metadata={(extended ? "extended" : "base")}");
    Console.WriteLine($"dirs={stats.Directories} files={stats.Files} symlinks={stats.Symlinks}");
    if (extended)
    {
        Console.WriteLine($"hardlinks={stats.Hardlinks} devices={stats.Devices} pipes={stats.Pipes}");
        Console.WriteLine($"size={stats.Size} usage={stats.Usage} bytes");
    }
    Console.WriteLine($"duration={stats.Duration:F6} seconds");
    PrintErrors(stats.Errors);
    return stats.Errors.Count == 0 ? 0 : 1;
}

static int Scan(string[] args)
{
    if (!TryRootAndExtended(args, ".", out string root, out bool extended))
    {
        Console.Error.WriteLine("Usage: examples scandir [ROOT] [--ext]");
        return 1;
    }

    ScanResult result = Scandir.Collect(root, new ScandirOptions
    {
        Sorted = true,
        Metadata = extended ? MetadataKind.Extended : MetadataKind.Basic,
    });
    Console.WriteLine($"root={root}, metadata={(extended ? "extended" : "base")}, entries={result.Entries.Count}");
    PrintErrors(result.Errors);
    foreach (ScandirEntry entry in result.Entries)
    {
        char type = entry.IsSymlink ? 'S' : entry.IsDirectory ? 'D' : entry.IsFile ? 'F' : '?';
        string line = $"{type} {entry.Path} size={entry.Size} ctime={entry.CTime:F3}";
        if (entry.HasExtendedMetadata)
        {
            line += $" mode={entry.Mode} ino={entry.Inode} uid={entry.UserId} gid={entry.GroupId}";
        }
        Console.WriteLine(line);
    }
    return result.Errors.Count == 0 ? 0 : 1;
}

static int Walk(string[] args)
{
    if (!TryRootAndExtended(args, "/usr", out string root, out bool extended))
    {
        Console.Error.WriteLine("Usage: examples walk [ROOT] [--ext]");
        return 1;
    }

    var options = new ScandirOptions
    {
        MaxFileCount = 100,
        Metadata = extended ? MetadataKind.Extended : MetadataKind.Basic,
    };
    Stopwatch timer = Stopwatch.StartNew();
    ScanResult result = Scandir.Collect(root, options);
    timer.Stop();
    Console.WriteLine($"root={root} entries={result.Entries.Count} errors={result.Errors.Count} duration={timer.Elapsed.TotalSeconds:F6} seconds");
    PrintErrors(result.Errors);
    foreach (ScandirEntry entry in result.Entries.Take(20))
    {
        char type = entry.IsSymlink ? 'S' : entry.IsDirectory ? 'D' : entry.IsFile ? 'F' : '?';
        Console.WriteLine($"{type} {entry.Path} size={entry.Size}");
    }
    if (result.Entries.Count > 20)
    {
        Console.WriteLine($"... ({result.Entries.Count} entries total; showing first 20)");
    }
    return result.Errors.Count == 0 ? 0 : 1;
}

static int Cifs(string[] args)
{
    if (args.Length > 1)
    {
        Console.Error.WriteLine("Usage: examples cifs [MOUNT_PATH]");
        return 1;
    }

    string root = args.Length == 1 ? args[0] : "/mnt/smb";
    ScanResult result = Scandir.Collect(root, new ScandirOptions { Metadata = MetadataKind.Extended });
    Console.WriteLine($"scanned {result.Entries.Count} entries, {result.Errors.Count} errors from {root}");
    int ctimeOk = 0;
    int ctimeNone = 0;
    int statChecks = 0;
    double maxSkew = 0;
    foreach (ScandirEntry entry in result.Entries.Where(entry => entry.IsFile))
    {
        if (entry.CTime > 0)
        {
            ++ctimeOk;
        }
        else
        {
            ++ctimeNone;
            Console.WriteLine($"  CTIME-ZERO {entry.Path,-30} <<< REGRESSION?");
        }

        double? statCtime = TryGetLinuxCtime(Path.Combine(root, entry.Path));
        double? skew = statCtime is null ? null : Math.Abs(entry.CTime - statCtime.Value);
        if (skew is not null)
        {
            ++statChecks;
            maxSkew = Math.Max(maxSkew, skew.Value);
        }
        string line = $"  {entry.Path,-30} ctime={entry.CTime:F6}";
        if (entry.HasExtendedMetadata)
        {
            line += $" mode={entry.Mode} ino={entry.Inode}";
        }
        if (skew is not null)
        {
            line += $" stat-skew={skew.Value:F6}s";
        }
        Console.WriteLine(line);
    }
    PrintErrors(result.Errors);
    Console.WriteLine($"summary: ctime {ctimeOk} nonzero / {ctimeNone} zero, stat checks={statChecks}, max ctime skew={maxSkew:F6}s");
    if (statChecks == 0)
    {
        Console.Error.WriteLine("warning: no entries could be checked against statx() (Linux only)");
    }
    return ctimeNone != 0 || result.Errors.Count != 0 ? 1 : 0;
}

static int Benchmark(string[] args)
{
    if (args.Length is < 1 or > 2)
    {
        Console.Error.WriteLine("Usage: examples benchmark ROOT [ITERATIONS]");
        return 1;
    }
    int iterations = 10;
    if (args.Length == 2 && (!int.TryParse(args[1], NumberStyles.None, CultureInfo.InvariantCulture, out iterations) || iterations < 1))
    {
        Console.Error.WriteLine("ITERATIONS must be a positive integer");
        return 1;
    }

    string[] names = ["collect", "collect-no-output", "count", "collect-ext"];
    for (int mode = 0; mode < names.Length; ++mode)
    {
        double elapsed = 0;
        int entries = 0;
        var options = new ScandirOptions { Metadata = mode == 3 ? MetadataKind.Extended : MetadataKind.Basic };
        for (int i = -1; i < iterations; ++i)
        {
            Stopwatch timer = Stopwatch.StartNew();
            if (mode == 2)
            {
                ScandirStatistics stats = Scandir.Count(args[0], options);
                entries = stats.Directories + stats.Files + stats.Symlinks;
            }
            else if (mode == 1)
            {
                Scandir.CollectDiscardingResults(args[0], options);
                entries = 0;
            }
            else
            {
                ScanResult result = Scandir.Collect(args[0], options);
                entries = result.Entries.Count;
            }
            timer.Stop();
            if (i >= 0)
            {
                elapsed += timer.Elapsed.TotalSeconds;
            }
        }
        Console.WriteLine($"{names[mode]}: {elapsed * 1000 / iterations:F3} ms/scan ({iterations} iterations, {entries} entries)");
    }
    return 0;
}

static bool TryRootAndExtended(string[] args, string defaultRoot, out string root, out bool extended)
{
    root = defaultRoot;
    extended = false;
    bool haveRoot = false;
    foreach (string arg in args)
    {
        if (arg == "--ext")
        {
            extended = true;
        }
        else if (!haveRoot)
        {
            root = arg;
            haveRoot = true;
        }
        else
        {
            return false;
        }
    }
    return true;
}

static void PrintErrors(IReadOnlyList<string> errors)
{
    foreach (string error in errors)
    {
        Console.Error.WriteLine($"scan error: {error}");
    }
}

static double? TryGetLinuxCtime(string path)
{
    if (!OperatingSystem.IsLinux())
    {
        return null;
    }
    try
    {
        var stat = new LinuxStatx();
        if (LinuxStatxNative.Statx(-100, path, 0, 0x7ff, ref stat) != 0)
        {
            return null;
        }
        return stat.ChangeTime.Seconds + stat.ChangeTime.Nanoseconds * 1e-9;
    }
    catch (DllNotFoundException)
    {
        return null;
    }
    catch (EntryPointNotFoundException)
    {
        return null;
    }
}

[StructLayout(LayoutKind.Sequential)]
struct LinuxStatxTimestamp
{
    internal long Seconds;
    internal uint Nanoseconds;
    internal int Reserved;
}

[StructLayout(LayoutKind.Sequential, Size = 256)]
struct LinuxStatx
{
    internal uint Mask;
    internal uint BlockSize;
    internal ulong Attributes;
    internal uint LinkCount;
    internal uint UserId;
    internal uint GroupId;
    internal ushort Mode;
    internal ushort Spare0;
    internal ulong Inode;
    internal ulong Size;
    internal ulong Blocks;
    internal ulong AttributesMask;
    internal LinuxStatxTimestamp AccessTime;
    internal LinuxStatxTimestamp BirthTime;
    internal LinuxStatxTimestamp ChangeTime;
    internal LinuxStatxTimestamp ModifyTime;
}

static class LinuxStatxNative
{
    [DllImport("libc", EntryPoint = "statx", CharSet = CharSet.Ansi, SetLastError = true)]
    internal static extern int Statx(int directory, [MarshalAs(UnmanagedType.LPUTF8Str)] string path, int flags, uint mask, ref LinuxStatx buffer);
}
