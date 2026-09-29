package com.scandir.options;

import com.scandir.ffm.ScandirLinker;

import java.lang.foreign.Arena;
import java.lang.foreign.GroupLayout;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.MemoryLayout.PathElement;
import java.lang.foreign.ValueLayout;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * Scan options configuration.
 * Mirrors the cscandir_options C struct.
 */
public final class Options {
    private boolean sorted = false;
    private boolean skipHidden = false;
    private long maxDepth = Long.MAX_VALUE;
    private long maxFileCnt = Long.MAX_VALUE;
    private final List<String> dirInclude = new ArrayList<>();
    private final List<String> dirExclude = new ArrayList<>();
    private final List<String> fileInclude = new ArrayList<>();
    private final List<String> fileExclude = new ArrayList<>();
    private boolean caseSensitive = false;
    private boolean followLinks = false;
    private ReturnType returnType = ReturnType.BASE;

    public Options() {
    }

    public static Options defaultOptions() {
        return new Options();
    }

    public Options sorted(boolean value) {
        this.sorted = value;
        return this;
    }

    public Options skipHidden(boolean value) {
        this.skipHidden = value;
        return this;
    }

    public Options maxDepth(long value) {
        this.maxDepth = value;
        return this;
    }

    public Options maxFileCnt(long value) {
        this.maxFileCnt = value;
        return this;
    }

    public Options dirInclude(String... patterns) {
        this.dirInclude.clear();
        for (String p : patterns) {
            this.dirInclude.add(p);
        }
        return this;
    }

    public Options dirExclude(String... patterns) {
        this.dirExclude.clear();
        for (String p : patterns) {
            this.dirExclude.add(p);
        }
        return this;
    }

    public Options fileInclude(String... patterns) {
        this.fileInclude.clear();
        for (String p : patterns) {
            this.fileInclude.add(p);
        }
        return this;
    }

    public Options fileExclude(String... patterns) {
        this.fileExclude.clear();
        for (String p : patterns) {
            this.fileExclude.add(p);
        }
        return this;
    }

    public Options caseSensitive(boolean value) {
        this.caseSensitive = value;
        return this;
    }

    public Options followLinks(boolean value) {
        this.followLinks = value;
        return this;
    }

    public Options returnType(ReturnType type) {
        this.returnType = type;
        return this;
    }

    public boolean isSorted() {
        return sorted;
    }

    public boolean isSkipHidden() {
        return skipHidden;
    }

    public long getMaxDepth() {
        return maxDepth;
    }

    public long getMaxFileCnt() {
        return maxFileCnt;
    }

    public List<String> getDirInclude() {
        return List.copyOf(dirInclude);
    }

    public List<String> getDirExclude() {
        return List.copyOf(dirExclude);
    }

    public List<String> getFileInclude() {
        return List.copyOf(fileInclude);
    }

    public List<String> getFileExclude() {
        return List.copyOf(fileExclude);
    }

    public boolean isCaseSensitive() {
        return caseSensitive;
    }

    public boolean isFollowLinks() {
        return followLinks;
    }

    public ReturnType getReturnType() {
        return returnType;
    }

    /**
     * Creates a native cscandir_options struct in the given arena.
     * The returned segment is valid for the lifetime of the arena.
     */
    public MemorySegment toNative(Arena arena) {
        GroupLayout layout = ScandirLinker.CSCANDIR_OPTIONS_LAYOUT;
        // Allocate the struct
        MemorySegment nativeOptions = arena.allocate(layout);

        // Set primitive fields
        nativeOptions.set(ValueLayout.JAVA_BYTE,
                layout.byteOffset(PathElement.groupElement("sorted")), sorted ? (byte) 1 : (byte) 0);
        nativeOptions.set(ValueLayout.JAVA_BYTE,
                layout.byteOffset(PathElement.groupElement("skip_hidden")), skipHidden ? (byte) 1 : (byte) 0);
        nativeOptions.set(ValueLayout.JAVA_LONG,
                layout.byteOffset(PathElement.groupElement("max_depth")), maxDepth);
        nativeOptions.set(ValueLayout.JAVA_LONG,
                layout.byteOffset(PathElement.groupElement("max_file_cnt")), maxFileCnt);
        nativeOptions.set(ValueLayout.JAVA_BYTE,
                layout.byteOffset(PathElement.groupElement("case_sensitive")), caseSensitive ? (byte) 1 : (byte) 0);
        nativeOptions.set(ValueLayout.JAVA_BYTE,
                layout.byteOffset(PathElement.groupElement("follow_links")), followLinks ? (byte) 1 : (byte) 0);
        nativeOptions.set(ValueLayout.JAVA_INT,
                layout.byteOffset(PathElement.groupElement("return_type")), returnType.ordinal());

        // Filter arrays - allocate and populate
        nativeOptions.set(ValueLayout.ADDRESS,
                layout.byteOffset(PathElement.groupElement("dir_include")), createStringArray(dirInclude, arena));
        nativeOptions.set(ValueLayout.JAVA_LONG,
                layout.byteOffset(PathElement.groupElement("dir_include_len")), (long) dirInclude.size());

        nativeOptions.set(ValueLayout.ADDRESS,
                layout.byteOffset(PathElement.groupElement("dir_exclude")), createStringArray(dirExclude, arena));
        nativeOptions.set(ValueLayout.JAVA_LONG,
                layout.byteOffset(PathElement.groupElement("dir_exclude_len")), (long) dirExclude.size());

        nativeOptions.set(ValueLayout.ADDRESS,
                layout.byteOffset(PathElement.groupElement("file_include")), createStringArray(fileInclude, arena));
        nativeOptions.set(ValueLayout.JAVA_LONG,
                layout.byteOffset(PathElement.groupElement("file_include_len")), (long) fileInclude.size());

        nativeOptions.set(ValueLayout.ADDRESS,
                layout.byteOffset(PathElement.groupElement("file_exclude")), createStringArray(fileExclude, arena));
        nativeOptions.set(ValueLayout.JAVA_LONG,
                layout.byteOffset(PathElement.groupElement("file_exclude_len")), (long) fileExclude.size());

        return nativeOptions;
    }

    private static MemorySegment createStringArray(List<String> strings, Arena arena) {
        if (strings.isEmpty()) {
            return MemorySegment.NULL;
        }

        // Allocate array of pointers
        var array = arena.allocate(ValueLayout.ADDRESS, strings.size());

        for (int i = 0; i < strings.size(); i++) {
            var str = allocateString(arena, strings.get(i).getBytes());
            array.setAtIndex(ValueLayout.ADDRESS, i, str);
        }

        return array;
    }

    private static MemorySegment allocateString(Arena arena, byte[] bytes) {
        MemorySegment seg = arena.allocate(bytes.length + 1);
        MemorySegment.copy(bytes, 0, seg, ValueLayout.JAVA_BYTE, 0, bytes.length);
        seg.set(ValueLayout.JAVA_BYTE, bytes.length, (byte) 0);
        return seg;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o)
            return true;
        if (!(o instanceof Options))
            return false;
        Options that = (Options) o;
        return sorted == that.sorted &&
                skipHidden == that.skipHidden &&
                maxDepth == that.maxDepth &&
                maxFileCnt == that.maxFileCnt &&
                caseSensitive == that.caseSensitive &&
                followLinks == that.followLinks &&
                returnType == that.returnType &&
                dirInclude.equals(that.dirInclude) &&
                dirExclude.equals(that.dirExclude) &&
                fileInclude.equals(that.fileInclude) &&
                fileExclude.equals(that.fileExclude);
    }

    @Override
    public int hashCode() {
        return Objects.hash(sorted, skipHidden, maxDepth, maxFileCnt,
                dirInclude, dirExclude, fileInclude, fileExclude,
                caseSensitive, followLinks, returnType);
    }
}