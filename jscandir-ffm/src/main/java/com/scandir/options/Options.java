package com.scandir.options;

import com.scandir.ffm.ScandirLinker;

import java.lang.foreign.Arena;
import java.lang.foreign.GroupLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.charset.StandardCharsets;
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
        if (value < 0) throw new IllegalArgumentException("maxDepth must be non-negative");
        this.maxDepth = value;
        return this;
    }

    public Options maxFileCnt(long value) {
        if (value < 0) throw new IllegalArgumentException("maxFileCnt must be non-negative");
        this.maxFileCnt = value;
        return this;
    }

    public Options dirInclude(String... patterns) {
        setPatterns(this.dirInclude, patterns);
        return this;
    }

    public Options dirExclude(String... patterns) {
        setPatterns(this.dirExclude, patterns);
        return this;
    }

    public Options fileInclude(String... patterns) {
        setPatterns(this.fileInclude, patterns);
        return this;
    }

    public Options fileExclude(String... patterns) {
        setPatterns(this.fileExclude, patterns);
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
        this.returnType = Objects.requireNonNull(type, "type");
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
        MemorySegment nativeOptions = arena.allocate(layout);
        nativeOptions.set(ValueLayout.JAVA_BYTE, ScandirLinker.offset(layout, "sorted"), (byte) (sorted ? 1 : 0));
        nativeOptions.set(ValueLayout.JAVA_BYTE, ScandirLinker.offset(layout, "skip_hidden"), (byte) (skipHidden ? 1 : 0));
        ScandirLinker.setSizeT(nativeOptions, ScandirLinker.offset(layout, "max_depth"), maxDepth);
        ScandirLinker.setSizeT(nativeOptions, ScandirLinker.offset(layout, "max_file_cnt"), maxFileCnt);
        nativeOptions.set(ValueLayout.JAVA_BYTE, ScandirLinker.offset(layout, "case_sensitive"), (byte) (caseSensitive ? 1 : 0));
        nativeOptions.set(ValueLayout.JAVA_BYTE, ScandirLinker.offset(layout, "follow_links"), (byte) (followLinks ? 1 : 0));
        nativeOptions.set(ValueLayout.JAVA_INT, ScandirLinker.offset(layout, "return_type"), returnType.getValue());
        setFilter(nativeOptions, layout, "dir_include", dirInclude, arena);
        setFilter(nativeOptions, layout, "dir_exclude", dirExclude, arena);
        setFilter(nativeOptions, layout, "file_include", fileInclude, arena);
        setFilter(nativeOptions, layout, "file_exclude", fileExclude, arena);
        return nativeOptions;
    }

    private static void setFilter(MemorySegment options, GroupLayout layout, String name,
                                  List<String> patterns, Arena arena) {
        MemorySegment values = MemorySegment.NULL;
        if (!patterns.isEmpty()) {
            values = arena.allocate(ValueLayout.ADDRESS, patterns.size());
            for (int i = 0; i < patterns.size(); i++) {
                String pattern = Objects.requireNonNull(patterns.get(i), "filter pattern");
                if (pattern.indexOf('\0') >= 0) throw new IllegalArgumentException("filter pattern contains NUL");
                values.setAtIndex(ValueLayout.ADDRESS, i, arena.allocateFrom(pattern, StandardCharsets.UTF_8));
            }
        }
        options.set(ValueLayout.ADDRESS, ScandirLinker.offset(layout, name), values);
        ScandirLinker.setSizeT(options, ScandirLinker.offset(layout, name + "_len"), patterns.size());
    }

    private static void setPatterns(List<String> destination, String[] patterns) {
        Objects.requireNonNull(patterns, "patterns");
        destination.clear();
        for (String pattern : patterns) destination.add(Objects.requireNonNull(pattern, "pattern"));
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