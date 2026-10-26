package com.scandir.options;

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