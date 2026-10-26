using CSharpScandir;

return TestSuite.Run();

static class TestSuite
{
    private static int _assertions;

    internal static int Run()
    {
        string root = Path.Combine(Path.GetTempPath(), $"csharpscandir-api-{Guid.NewGuid():N}");
        Directory.CreateDirectory(Path.Combine(root, "included"));
        Directory.CreateDirectory(Path.Combine(root, "excluded"));
        string[] files = ["a.txt", "b.TXT", ".hidden", "included/kept.txt", "excluded/omitted.txt"];
        foreach (string file in files)
        {
            string path = Path.Combine(root, file);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, "abc");
        }

        try
        {
            TestInvalidInputs(root);
            TestCollectAndCount(root);
            TestFilters(root);
            Console.WriteLine($"C# API regression tests passed ({_assertions} assertions)");
            return 0;
        }
        finally
        {
            Directory.Delete(root, recursive: true);
        }
    }

    private static void TestInvalidInputs(string root)
    {
        Throws<ArgumentNullException>(() => Scandir.Collect(null!));
        ScandirException missingRoot = Throws<ScandirException>(() => Scandir.Collect("/definitely/not/a/csharpscandir/test/path"));
        Equal(ScandirErrorCode.ScanFailed, missingRoot.Code);
        Throws<ArgumentOutOfRangeException>(() => Scandir.Collect(root,
            new ScandirOptions { Metadata = (MetadataKind)99 }));
        ScandirException badGlob = Throws<ScandirException>(() => Scandir.Collect(root,
            new ScandirOptions { FileInclude = ["["] }));
        Equal(ScandirErrorCode.InvalidArgument, badGlob.Code);
        ScandirException badCountGlob = Throws<ScandirException>(() => Scandir.Count(root,
            new ScandirOptions { DirectoryExclude = ["["] }));
        Equal(ScandirErrorCode.InvalidArgument, badCountGlob.Code);
        Throws<ArgumentException>(() => Scandir.Collect("root\0suffix"));
        Throws<ArgumentException>(() => Scandir.Collect(root,
            new ScandirOptions { FileInclude = ["bad\0filter"] }));
        Throws<ArgumentNullException>(() => Scandir.Collect(root,
            new ScandirOptions { FileInclude = null! }));
    }

    private static void TestCollectAndCount(string root)
    {
        ScanResult basic = Scandir.Collect(root);
        Equal(8, basic.Entries.Count);
        Equal(0, basic.Errors.Count);
        True(basic.Entries.Any(entry => entry.Path == ".hidden"));
        True(basic.Entries.Any(entry => entry.Path == "included/kept.txt"));
        True(basic.Entries.All(entry => !entry.HasExtendedMetadata));

        var extendedOptions = new ScandirOptions { Sorted = true, Metadata = MetadataKind.Extended };
        ScanResult extended = Scandir.Collect(root, extendedOptions);
        Equal(8, extended.Entries.Count);
        Equal(Path.GetFileName(root), extended.Entries[0].Path);
        string[] expected = [".hidden", "a.txt", "b.TXT", "excluded", "excluded/omitted.txt", "included", "included/kept.txt"];
        SequenceEqual(expected, extended.Entries.Skip(1).Select(entry => entry.Path));
        foreach (ScandirEntry entry in extended.Entries.Skip(1))
        {
            True(entry.HasExtendedMetadata);
            True(entry.Mode != 0);
            True(entry.LinkCount > 0);
            True(entry.CTime > 0 && entry.MTime > 0 && entry.ATime > 0);
            if (entry.IsFile)
            {
                Equal(3UL, entry.Size);
            }
        }

        ScandirStatistics stats = Scandir.Count(root, extendedOptions);
        Equal(2, stats.Directories);
        Equal(5, stats.Files);
        Equal(0, stats.Symlinks);
        True(stats.Size >= 15);
        True(stats.Duration >= 0);
        Equal(0, stats.Errors.Count);
        Scandir.CollectDiscardingResults(root);

        for (int i = 0; i < 30; ++i)
        {
            Equal(8, Scandir.Collect(root).Entries.Count);
            Equal(5, Scandir.Count(root).Files);
        }
    }

    private static void TestFilters(string root)
    {
        var options = new ScandirOptions
        {
            SkipHidden = true,
            DirectoryInclude = ["included/**"],
            DirectoryExclude = ["excluded/**"],
            FileInclude = ["*.txt"],
            CaseSensitive = true,
        };
        ScanResult filtered = Scandir.Collect(root, options);
        Equal(4, filtered.Entries.Count);
        Contains(filtered.Entries, "a.txt");
        Contains(filtered.Entries, "included");
        Contains(filtered.Entries, "included/kept.txt");
        True(!filtered.Entries.Any(entry => entry.Path == ".hidden"));

        options.FileExclude = ["kept*"];
        filtered = Scandir.Collect(root, options);
        Equal(3, filtered.Entries.Count);
        True(!filtered.Entries.Any(entry => entry.Path == "included/kept.txt"));

        options = new ScandirOptions { MaxDepth = 1 };
        Equal(6, Scandir.Collect(root, options).Entries.Count);
        options.MaxDepth = 0;
        options.MaxFileCount = 1;
        int limitedCount = Scandir.Collect(root, options).Entries.Count;
        True(limitedCount > 0 && limitedCount < 8);
    }

    private static void Contains(IReadOnlyList<ScandirEntry> entries, string path)
    {
        True(entries.Any(entry => entry.Path == path), $"Expected path {path}");
    }

    private static void True(bool condition, string? message = null)
    {
        ++_assertions;
        if (!condition)
        {
            throw new InvalidOperationException(message ?? "Expected condition to be true.");
        }
    }

    private static void Equal<T>(T expected, T actual)
    {
        ++_assertions;
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new InvalidOperationException($"Expected {expected}, got {actual}.");
        }
    }

    private static void SequenceEqual<T>(IEnumerable<T> expected, IEnumerable<T> actual)
    {
        ++_assertions;
        if (!expected.SequenceEqual(actual))
        {
            throw new InvalidOperationException($"Expected [{string.Join(", ", expected)}], got [{string.Join(", ", actual)}].");
        }
    }

    private static TException Throws<TException>(Action action) where TException : Exception
    {
        ++_assertions;
        try
        {
            action();
        }
        catch (TException exception)
        {
            return exception;
        }
        throw new InvalidOperationException($"Expected {typeof(TException).Name}.");
    }
}
