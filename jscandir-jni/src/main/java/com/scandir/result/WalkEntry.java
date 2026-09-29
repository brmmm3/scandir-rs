package com.scandir.result;

/**
 * A walk entry - one directory visited by Walk, with its Toc.
 */
public final class WalkEntry {
    private final String dir;
    private final Toc toc;

    // Package-private constructor for JNI
    WalkEntry(String dir, Toc toc) {
        this.dir = dir;
        this.toc = toc;
    }

    public String getDir() { return dir; }
    public Toc getToc() { return toc; }

    @Override
    public String toString() {
        return String.format("WalkEntry{dir='%s', toc=%s}", dir, toc);
    }
}