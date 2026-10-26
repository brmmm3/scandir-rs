package com.scandir;

import com.scandir.error.JScandirError;
import com.scandir.ffm.ScandirLinker;
import com.scandir.options.Options;
import com.scandir.result.CollectResult;
import com.scandir.result.Entry;
import com.scandir.result.Statistics;
import com.scandir.result.Toc;
import com.scandir.result.WalkEntry;
import com.scandir.result.WalkResult;

import java.lang.foreign.Arena;
import java.lang.foreign.GroupLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.file.Path;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.TreeMap;

/** Java API over the cscandir C ABI using the Foreign Function & Memory API. */
public final class JScandir {
    private JScandir() { }

    public static CollectResult collect(Path root, Options options) {
        return collect(Objects.requireNonNull(root, "root").toString(), options);
    }

    public static CollectResult collect(String root, Options options) {
        Objects.requireNonNull(root, "root");
        if (root.indexOf('\0') >= 0) throw JScandirError.invalidArgument("root path contains NUL");
        long started = System.nanoTime();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cOptions = options == null ? MemorySegment.NULL : options.toNative(arena);
            MemorySegment cRoot = arena.allocateFrom(root);
            MemorySegment entries = arena.allocate(ScandirLinker.CSCANDIR_ENTRY_LIST_LAYOUT);
            MemorySegment errors = arena.allocate(ScandirLinker.CSCANDIR_STRING_LIST_LAYOUT);
            MemorySegment error = arena.allocate(ScandirLinker.CSCANDIR_ERROR_LAYOUT);
            entries.fill((byte) 0);
            errors.fill((byte) 0);
            error.fill((byte) 0);
            int status = (int) ScandirLinker.COLLECT.invokeExact(cRoot, cOptions, entries, errors, error);
            try {
                String nativeMessage = ScandirLinker.readCString(error.get(ValueLayout.ADDRESS,
                        ScandirLinker.offset(ScandirLinker.CSCANDIR_ERROR_LAYOUT, "message")));
                if (status != 0) throw JScandirError.fromNative(status,
                        nativeMessage == null ? "collect failed" : nativeMessage);
                List<Entry> convertedEntries = readEntries(entries);
                List<String> convertedErrors = readStrings(errors);
                int firstCounted = hasSyntheticRoot(root, convertedEntries) ? 1 : 0;
                Statistics stats = Statistics.fromEntries(convertedEntries.subList(firstCounted, convertedEntries.size()),
                        (System.nanoTime() - started) / 1_000_000_000.0);
                return new CollectResult(convertedEntries, convertedErrors, stats);
            } finally {
                ScandirLinker.FREE_ENTRY_LIST.invokeExact(entries);
                ScandirLinker.FREE_STRING_LIST.invokeExact(errors);
                ScandirLinker.FREE_ERROR.invokeExact(error);
            }
        } catch (JScandirError e) {
            throw e;
        } catch (Throwable e) {
            throw new JScandirError("Native collect call failed: " + e.getMessage(), e);
        }
    }

    public static Statistics count(Path root, Options options) {
        return count(Objects.requireNonNull(root, "root").toString(), options);
    }

    public static Statistics count(String root, Options options) {
        Objects.requireNonNull(root, "root");
        if (root.indexOf('\0') >= 0) throw JScandirError.invalidArgument("root path contains NUL");
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cOptions = options == null ? MemorySegment.NULL : options.toNative(arena);
            MemorySegment cRoot = arena.allocateFrom(root);
            MemorySegment stats = arena.allocate(ScandirLinker.CSCANDIR_STATISTICS_LAYOUT);
            MemorySegment errors = arena.allocate(ScandirLinker.CSCANDIR_STRING_LIST_LAYOUT);
            MemorySegment error = arena.allocate(ScandirLinker.CSCANDIR_ERROR_LAYOUT);
            stats.fill((byte) 0);
            errors.fill((byte) 0);
            error.fill((byte) 0);
            int status = (int) ScandirLinker.COUNT.invokeExact(cRoot, cOptions, stats, errors, error);
            try {
                String nativeMessage = ScandirLinker.readCString(error.get(ValueLayout.ADDRESS,
                        ScandirLinker.offset(ScandirLinker.CSCANDIR_ERROR_LAYOUT, "message")));
                if (status != 0) throw JScandirError.fromNative(status,
                        nativeMessage == null ? "count failed" : nativeMessage);
                Statistics nativeStats = readStatistics(stats, readStrings(errors));
                return new Statistics(nativeStats.getDirs(), nativeStats.getFiles(), nativeStats.getSlinks(),
                        nativeStats.getHlinks(), nativeStats.getDevices(), nativeStats.getPipes(),
                        nativeStats.getSize(), nativeStats.getUsage(), nativeStats.getDuration(), nativeStats.getErrors());
            } finally {
                ScandirLinker.FREE_STRING_LIST.invokeExact(errors);
                ScandirLinker.FREE_ERROR.invokeExact(error);
            }
        } catch (JScandirError e) {
            throw e;
        } catch (Throwable e) {
            throw new JScandirError("Native count call failed: " + e.getMessage(), e);
        }
    }

    /** Groups a single synchronous collect result by parent directory. */
    public static WalkResult walk(Path root, Options options) {
        return walk(Objects.requireNonNull(root, "root").toString(), options);
    }

    public static WalkResult walk(String root, Options options) {
        CollectResult collected = collect(root, options);
        Map<String, TocParts> grouped = new TreeMap<>();
        TocParts merged = new TocParts();
        List<Entry> entries = collected.getEntries();
        int firstCounted = hasSyntheticRoot(root, entries) ? 1 : 0;
        for (int i = firstCounted; i < entries.size(); i++) {
            Entry entry = entries.get(i);
            String path = entry.getPath();
            int separator = path.lastIndexOf('/');
            String parent = separator < 0 ? "" : path.substring(0, separator);
            String name = separator < 0 ? path : path.substring(separator + 1);
            TocParts part = grouped.computeIfAbsent(parent, ignored -> new TocParts());
            if (entry.isDir()) { part.dirs.add(name); merged.dirs.add(path); }
            else if (entry.isFile()) { part.files.add(name); merged.files.add(path); }
            else if (entry.isSymlink()) { part.symlinks.add(name); merged.symlinks.add(path); }
            else { part.other.add(name); merged.other.add(path); }
        }
        merged.errors.addAll(collected.getErrors());
        List<WalkEntry> roots = new ArrayList<>(grouped.size());
        grouped.forEach((dir, part) -> roots.add(new WalkEntry(dir, part.toToc())));
        return new WalkResult(merged.toToc(), roots, collected.getStatistics());
    }

    private static boolean hasSyntheticRoot(String root, List<Entry> entries) {
        if (entries.isEmpty()) return false;
        Entry first = entries.get(0);
        Path input = Path.of(root).normalize();
        Path normalized = input.isAbsolute() ? input : Path.of("").toAbsolutePath().resolve(input).normalize();
        try {
            normalized = normalized.toRealPath();
        } catch (IOException ignored) {
            // Keep the normalized absolute path for a root that cannot be resolved.
        }
        Path basename = normalized.getFileName();
        return first.isDir() && basename != null && first.getPath().equals(basename.toString());
    }

    private static List<Entry> readEntries(MemorySegment list) {
        long len = ScandirLinker.getSizeT(list,
                ScandirLinker.offset(ScandirLinker.CSCANDIR_ENTRY_LIST_LAYOUT, "len"));
        if (len > Integer.MAX_VALUE) throw new IllegalStateException("entry count exceeds Java list capacity");
        if (len == 0) return List.of();
        MemorySegment pointer = list.get(ValueLayout.ADDRESS,
                ScandirLinker.offset(ScandirLinker.CSCANDIR_ENTRY_LIST_LAYOUT, "entries"));
        List<Entry> result = new ArrayList<>((int) len);
        GroupLayout layout = ScandirLinker.CSCANDIR_ENTRY_LAYOUT;
        long stride = layout.byteSize();
        MemorySegment entries = pointer.reinterpret(Math.multiplyExact(len, stride));
        long pathOffset = ScandirLinker.offset(layout, "path");
        long symlinkOffset = ScandirLinker.offset(layout, "is_symlink");
        long dirOffset = ScandirLinker.offset(layout, "is_dir");
        long fileOffset = ScandirLinker.offset(layout, "is_file");
        long ctimeOffset = ScandirLinker.offset(layout, "ctime");
        long mtimeOffset = ScandirLinker.offset(layout, "mtime");
        long atimeOffset = ScandirLinker.offset(layout, "atime");
        long sizeOffset = ScandirLinker.offset(layout, "size");
        long hasExtOffset = ScandirLinker.offset(layout, "has_ext");
        long modeOffset = ScandirLinker.offset(layout, "mode");
        long inoOffset = ScandirLinker.offset(layout, "ino");
        long devOffset = ScandirLinker.offset(layout, "dev");
        long nlinkOffset = ScandirLinker.offset(layout, "nlink");
        long blksizeOffset = ScandirLinker.offset(layout, "blksize");
        long blocksOffset = ScandirLinker.offset(layout, "blocks");
        long uidOffset = ScandirLinker.offset(layout, "uid");
        long gidOffset = ScandirLinker.offset(layout, "gid");
        long rdevOffset = ScandirLinker.offset(layout, "rdev");
        for (long i = 0; i < len; i++) {
            long base = i * stride;
            String path = ScandirLinker.readCString(entries.get(ValueLayout.ADDRESS, base + pathOffset));
            result.add(new Entry(path,
                    entries.get(ValueLayout.JAVA_BYTE, base + symlinkOffset) != 0,
                    entries.get(ValueLayout.JAVA_BYTE, base + dirOffset) != 0,
                    entries.get(ValueLayout.JAVA_BYTE, base + fileOffset) != 0,
                    entries.get(ValueLayout.JAVA_DOUBLE, base + ctimeOffset),
                    entries.get(ValueLayout.JAVA_DOUBLE, base + mtimeOffset),
                    entries.get(ValueLayout.JAVA_DOUBLE, base + atimeOffset),
                    entries.get(ValueLayout.JAVA_LONG, base + sizeOffset),
                    entries.get(ValueLayout.JAVA_BYTE, base + hasExtOffset) != 0,
                    entries.get(ValueLayout.JAVA_INT, base + modeOffset),
                    entries.get(ValueLayout.JAVA_LONG, base + inoOffset),
                    entries.get(ValueLayout.JAVA_LONG, base + devOffset),
                    entries.get(ValueLayout.JAVA_LONG, base + nlinkOffset),
                    entries.get(ValueLayout.JAVA_LONG, base + blksizeOffset),
                    entries.get(ValueLayout.JAVA_LONG, base + blocksOffset),
                    entries.get(ValueLayout.JAVA_INT, base + uidOffset),
                    entries.get(ValueLayout.JAVA_INT, base + gidOffset),
                    entries.get(ValueLayout.JAVA_LONG, base + rdevOffset)));
        }
        return result;
    }

    private static List<String> readStrings(MemorySegment list) {
        long len = ScandirLinker.getSizeT(list,
                ScandirLinker.offset(ScandirLinker.CSCANDIR_STRING_LIST_LAYOUT, "len"));
        if (len > Integer.MAX_VALUE) throw new IllegalStateException("error count exceeds Java list capacity");
        if (len == 0) return List.of();
        MemorySegment pointer = list.get(ValueLayout.ADDRESS,
                ScandirLinker.offset(ScandirLinker.CSCANDIR_STRING_LIST_LAYOUT, "items"));
        MemorySegment items = pointer.reinterpret(Math.multiplyExact(len, ValueLayout.ADDRESS.byteSize()));
        List<String> result = new ArrayList<>((int) len);
        for (long i = 0; i < len; i++) {
            MemorySegment item = items.getAtIndex(ValueLayout.ADDRESS, i);
            result.add(ScandirLinker.readCString(item));
        }
        return result;
    }

    private static Statistics readStatistics(MemorySegment stats, List<String> errors) {
        GroupLayout layout = ScandirLinker.CSCANDIR_STATISTICS_LAYOUT;
        return new Statistics(
                stats.get(ValueLayout.JAVA_INT, ScandirLinker.offset(layout, "dirs")),
                stats.get(ValueLayout.JAVA_INT, ScandirLinker.offset(layout, "files")),
                stats.get(ValueLayout.JAVA_INT, ScandirLinker.offset(layout, "slinks")),
                stats.get(ValueLayout.JAVA_INT, ScandirLinker.offset(layout, "hlinks")),
                stats.get(ValueLayout.JAVA_INT, ScandirLinker.offset(layout, "devices")),
                stats.get(ValueLayout.JAVA_INT, ScandirLinker.offset(layout, "pipes")),
                stats.get(ValueLayout.JAVA_LONG, ScandirLinker.offset(layout, "size")),
                stats.get(ValueLayout.JAVA_LONG, ScandirLinker.offset(layout, "usage")),
                stats.get(ValueLayout.JAVA_DOUBLE, ScandirLinker.offset(layout, "duration")), errors);
    }

    private static final class TocParts {
        final List<String> dirs = new ArrayList<>(), files = new ArrayList<>();
        final List<String> symlinks = new ArrayList<>(), other = new ArrayList<>(), errors = new ArrayList<>();
        Toc toToc() { return new Toc(dirs, files, symlinks, other, errors); }
    }
}