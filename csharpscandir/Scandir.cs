using System.Runtime.InteropServices;

namespace CSharpScandir;

public static unsafe class Scandir
{
    public static ScanResult Collect(string root, ScandirOptions? options = null)
    {
        ArgumentNullException.ThrowIfNull(root);
        options ??= new ScandirOptions();

        using var nativeOptions = new NativeOptionsLease(options);
        IntPtr rootPointer = NativeMethods.AllocateUtf8(root);
        NativeMethods.NativeEntryList entries = default;
        NativeMethods.NativeStringList errors = default;
        NativeMethods.NativeError error = default;
        try
        {
            int status = NativeMethods.Collect(
                rootPointer,
                nativeOptions.Pointer,
                (IntPtr)(&entries),
                (IntPtr)(&errors),
                (IntPtr)(&error));
            ThrowIfFailed(status, error);
            return new ScanResult(ReadEntries(entries), ReadStrings(errors));
        }
        finally
        {
            NativeMethods.FreeEntryList((IntPtr)(&entries));
            NativeMethods.FreeStringList((IntPtr)(&errors));
            NativeMethods.FreeError((IntPtr)(&error));
            Marshal.FreeCoTaskMem(rootPointer);
        }
    }

    /// <summary>Run the scan while asking the native API not to marshal entries or warnings.</summary>
    public static void CollectDiscardingResults(string root, ScandirOptions? options = null)
    {
        ArgumentNullException.ThrowIfNull(root);
        options ??= new ScandirOptions();
        using var nativeOptions = new NativeOptionsLease(options);
        IntPtr rootPointer = NativeMethods.AllocateUtf8(root);
        NativeMethods.NativeError error = default;
        try
        {
            int status = NativeMethods.Collect(rootPointer, nativeOptions.Pointer, IntPtr.Zero, IntPtr.Zero, (IntPtr)(&error));
            ThrowIfFailed(status, error);
        }
        finally
        {
            NativeMethods.FreeError((IntPtr)(&error));
            Marshal.FreeCoTaskMem(rootPointer);
        }
    }

    public static ScandirStatistics Count(string root, ScandirOptions? options = null)
    {
        ArgumentNullException.ThrowIfNull(root);
        options ??= new ScandirOptions();
        using var nativeOptions = new NativeOptionsLease(options);
        IntPtr rootPointer = NativeMethods.AllocateUtf8(root);
        NativeMethods.NativeStatistics statistics = default;
        NativeMethods.NativeStringList errors = default;
        NativeMethods.NativeError error = default;
        try
        {
            int status = NativeMethods.Count(
                rootPointer,
                nativeOptions.Pointer,
                (IntPtr)(&statistics),
                (IntPtr)(&errors),
                (IntPtr)(&error));
            ThrowIfFailed(status, error);
            return new ScandirStatistics(
                statistics.Directories,
                statistics.Files,
                statistics.Symlinks,
                statistics.Hardlinks,
                statistics.Devices,
                statistics.Pipes,
                statistics.Size,
                statistics.Usage,
                statistics.Duration,
                ReadStrings(errors));
        }
        finally
        {
            NativeMethods.FreeStringList((IntPtr)(&errors));
            NativeMethods.FreeError((IntPtr)(&error));
            Marshal.FreeCoTaskMem(rootPointer);
        }
    }

    private static void ThrowIfFailed(int status, NativeMethods.NativeError error)
    {
        if (status != 0)
        {
            throw new ScandirException(status, error.Message == IntPtr.Zero
                ? "Unknown scandir error."
                : NativeMethods.ReadUtf8(error.Message));
        }
    }

    private static unsafe IReadOnlyList<ScandirEntry> ReadEntries(NativeMethods.NativeEntryList list)
    {
        var entries = new ScandirEntry[checked((int)list.Length)];
        NativeMethods.NativeEntry* nativeEntries = (NativeMethods.NativeEntry*)list.Entries;
        for (int i = 0; i < entries.Length; ++i)
        {
            NativeMethods.NativeEntry entry = nativeEntries[i];
            entries[i] = new ScandirEntry(
                NativeMethods.ReadUtf8(entry.Path),
                entry.IsSymlink != 0,
                entry.IsDirectory != 0,
                entry.IsFile != 0,
                entry.CTime,
                entry.MTime,
                entry.ATime,
                entry.Size,
                entry.HasExtendedMetadata != 0,
                entry.Mode,
                entry.Inode,
                entry.Device,
                entry.LinkCount,
                entry.BlockSize,
                entry.Blocks,
                entry.UserId,
                entry.GroupId,
                entry.SpecialDevice);
        }
        return entries;
    }

    private static unsafe IReadOnlyList<string> ReadStrings(NativeMethods.NativeStringList list)
    {
        var strings = new string[checked((int)list.Length)];
        IntPtr* nativeStrings = (IntPtr*)list.Items;
        for (int i = 0; i < strings.Length; ++i)
        {
            strings[i] = NativeMethods.ReadUtf8(nativeStrings[i]);
        }
        return strings;
    }

    private sealed unsafe class NativeOptionsLease : IDisposable
    {
        private readonly List<IntPtr> _arrays = [];
        private readonly List<IntPtr> _strings = [];
        internal IntPtr Pointer { get; private set; }

        internal NativeOptionsLease(ScandirOptions options)
        {
            try
            {
                if (!Enum.IsDefined(options.Metadata))
                {
                    throw new ArgumentOutOfRangeException(nameof(options), "Metadata must be Basic or Extended.");
                }

                Pointer = Marshal.AllocHGlobal(Marshal.SizeOf<NativeMethods.NativeOptions>());
                NativeMethods.OptionsInit(Pointer);
                NativeMethods.NativeOptions native = Marshal.PtrToStructure<NativeMethods.NativeOptions>(Pointer);
                native.Sorted = options.Sorted ? (byte)1 : (byte)0;
                native.SkipHidden = options.SkipHidden ? (byte)1 : (byte)0;
                native.MaxDepth = options.MaxDepth;
                native.MaxFileCount = options.MaxFileCount;
                native.CaseSensitive = options.CaseSensitive ? (byte)1 : (byte)0;
                native.FollowLinks = options.FollowLinks ? (byte)1 : (byte)0;
                native.ReturnType = (uint)options.Metadata;
                SetStrings(options.DirectoryInclude, out native.DirInclude, out native.DirIncludeLength);
                SetStrings(options.DirectoryExclude, out native.DirExclude, out native.DirExcludeLength);
                SetStrings(options.FileInclude, out native.FileInclude, out native.FileIncludeLength);
                SetStrings(options.FileExclude, out native.FileExclude, out native.FileExcludeLength);
                Marshal.StructureToPtr(native, Pointer, false);
            }
            catch
            {
                Dispose();
                throw;
            }
        }

        private void SetStrings(IReadOnlyList<string> values, out IntPtr pointer, out nuint length)
        {
            ArgumentNullException.ThrowIfNull(values);
            foreach (string value in values)
            {
                ArgumentNullException.ThrowIfNull(value);
                if (value.Contains('\0'))
                {
                    throw new ArgumentException("Filter strings cannot contain a NUL character.", nameof(values));
                }
            }
            length = checked((nuint)values.Count);
            if (values.Count == 0)
            {
                pointer = IntPtr.Zero;
                return;
            }

            pointer = Marshal.AllocHGlobal(checked(IntPtr.Size * values.Count));
            _arrays.Add(pointer);
            for (int i = 0; i < values.Count; ++i)
            {
                IntPtr item = NativeMethods.AllocateUtf8(values[i]);
                _strings.Add(item);
                Marshal.WriteIntPtr(pointer, i * IntPtr.Size, item);
            }
        }

        public void Dispose()
        {
            foreach (IntPtr allocation in _strings)
            {
                if (allocation != IntPtr.Zero)
                {
                    Marshal.FreeCoTaskMem(allocation);
                }
            }
            foreach (IntPtr allocation in _arrays)
            {
                if (allocation != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(allocation);
                }
            }
            _strings.Clear();
            _arrays.Clear();
            if (Pointer != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(Pointer);
                Pointer = IntPtr.Zero;
            }
        }
    }
}
