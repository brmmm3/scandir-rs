using System.Reflection;
using System.Runtime.InteropServices;

namespace CSharpScandir;

internal static class NativeMethods
{
    private const string LibraryName = "cscandir";

    static NativeMethods()
    {
        NativeLibrary.SetDllImportResolver(typeof(NativeMethods).Assembly, ResolveLibrary);
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct NativeOptions
    {
        internal byte Sorted;
        internal byte SkipHidden;
        internal nuint MaxDepth;
        internal nuint MaxFileCount;
        internal IntPtr DirInclude;
        internal nuint DirIncludeLength;
        internal IntPtr DirExclude;
        internal nuint DirExcludeLength;
        internal IntPtr FileInclude;
        internal nuint FileIncludeLength;
        internal IntPtr FileExclude;
        internal nuint FileExcludeLength;
        internal byte CaseSensitive;
        internal byte FollowLinks;
        internal uint ReturnType;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct NativeEntry
    {
        internal IntPtr Path;
        internal byte IsSymlink;
        internal byte IsDirectory;
        internal byte IsFile;
        internal double CTime;
        internal double MTime;
        internal double ATime;
        internal ulong Size;
        internal byte HasExtendedMetadata;
        internal uint Mode;
        internal ulong Inode;
        internal ulong Device;
        internal ulong LinkCount;
        internal ulong BlockSize;
        internal ulong Blocks;
        internal uint UserId;
        internal uint GroupId;
        internal ulong SpecialDevice;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct NativeEntryList
    {
        internal IntPtr Entries;
        internal nuint Length;
        internal nuint Capacity;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct NativeStringList
    {
        internal IntPtr Items;
        internal nuint Length;
        internal nuint Capacity;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct NativeError
    {
        internal int Code;
        internal IntPtr Message;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct NativeStatistics
    {
        internal int Directories;
        internal int Files;
        internal int Symlinks;
        internal int Hardlinks;
        internal int Devices;
        internal int Pipes;
        internal ulong Size;
        internal ulong Usage;
        internal double Duration;
    }

    [DllImport(LibraryName, EntryPoint = "cscandir_options_init", CallingConvention = CallingConvention.Cdecl)]
    internal static extern void OptionsInit(IntPtr options);

    [DllImport(LibraryName, EntryPoint = "cscandir_collect", CallingConvention = CallingConvention.Cdecl)]
    internal static extern int Collect(IntPtr root, IntPtr options, IntPtr entries, IntPtr errors, IntPtr error);

    [DllImport(LibraryName, EntryPoint = "cscandir_count", CallingConvention = CallingConvention.Cdecl)]
    internal static extern int Count(IntPtr root, IntPtr options, IntPtr statistics, IntPtr errors, IntPtr error);

    [DllImport(LibraryName, EntryPoint = "cscandir_free_entry_list", CallingConvention = CallingConvention.Cdecl)]
    internal static extern void FreeEntryList(IntPtr entries);

    [DllImport(LibraryName, EntryPoint = "cscandir_free_string_list", CallingConvention = CallingConvention.Cdecl)]
    internal static extern void FreeStringList(IntPtr strings);

    [DllImport(LibraryName, EntryPoint = "cscandir_free_error", CallingConvention = CallingConvention.Cdecl)]
    internal static extern void FreeError(IntPtr error);

    internal static IntPtr AllocateUtf8(string value)
    {
        ArgumentNullException.ThrowIfNull(value);
        if (value.Contains('\0'))
        {
            throw new ArgumentException("Native strings cannot contain a NUL character.", nameof(value));
        }
        return Marshal.StringToCoTaskMemUTF8(value);
    }

    internal static string ReadUtf8(IntPtr value)
    {
        return Marshal.PtrToStringUTF8(value) ?? string.Empty;
    }

    private static IntPtr ResolveLibrary(string libraryName, Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (libraryName != LibraryName)
        {
            return IntPtr.Zero;
        }

        string? configuredPath = Environment.GetEnvironmentVariable("SCANDIR_NATIVE_LIBRARY");
        if (!string.IsNullOrWhiteSpace(configuredPath) && NativeLibrary.TryLoad(configuredPath, out IntPtr configured))
        {
            return configured;
        }

        string fileName = OperatingSystem.IsWindows()
            ? "cscandir.dll"
            : OperatingSystem.IsMacOS() ? "libcscandir.dylib" : "libcscandir.so";
        for (DirectoryInfo? directory = new(AppContext.BaseDirectory); directory is not null; directory = directory.Parent)
        {
            foreach (string profile in new[] { "release", "debug" })
            {
                string candidate = Path.Combine(directory.FullName, "target", profile, fileName);
                if (NativeLibrary.TryLoad(candidate, out IntPtr handle))
                {
                    return handle;
                }
            }
        }

        return IntPtr.Zero;
    }
}
