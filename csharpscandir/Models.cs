namespace CSharpScandir;

public enum MetadataKind : uint
{
    Basic = 0,
    Extended = 1,
}

public enum ScandirErrorCode
{
    InvalidArgument = 1,
    InvalidUtf8 = 2,
    ScanFailed = 3,
    OutputContainsNul = 4,
    Panic = 5,
}

public sealed class ScandirOptions
{
    public bool Sorted { get; set; }
    public bool SkipHidden { get; set; }
    public nuint MaxDepth { get; set; } = nuint.MaxValue;
    public nuint MaxFileCount { get; set; } = nuint.MaxValue;
    public IReadOnlyList<string> DirectoryInclude { get; set; } = [];
    public IReadOnlyList<string> DirectoryExclude { get; set; } = [];
    public IReadOnlyList<string> FileInclude { get; set; } = [];
    public IReadOnlyList<string> FileExclude { get; set; } = [];
    public bool CaseSensitive { get; set; }
    public bool FollowLinks { get; set; }
    public MetadataKind Metadata { get; set; }
}

public sealed record ScandirEntry(
    string Path,
    bool IsSymlink,
    bool IsDirectory,
    bool IsFile,
    double CTime,
    double MTime,
    double ATime,
    ulong Size,
    bool HasExtendedMetadata,
    uint Mode,
    ulong Inode,
    ulong Device,
    ulong LinkCount,
    ulong BlockSize,
    ulong Blocks,
    uint UserId,
    uint GroupId,
    ulong SpecialDevice);

public sealed record ScanResult(IReadOnlyList<ScandirEntry> Entries, IReadOnlyList<string> Errors);

public sealed record ScandirStatistics(
    int Directories,
    int Files,
    int Symlinks,
    int Hardlinks,
    int Devices,
    int Pipes,
    ulong Size,
    ulong Usage,
    double Duration,
    IReadOnlyList<string> Errors);

public sealed class ScandirException : Exception
{
    public ScandirErrorCode Code { get; }

    internal ScandirException(int code, string message)
        : base(message)
    {
        Code = Enum.IsDefined(typeof(ScandirErrorCode), code)
            ? (ScandirErrorCode)code
            : ScandirErrorCode.ScanFailed;
    }
}
