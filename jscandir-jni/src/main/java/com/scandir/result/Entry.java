package com.scandir.result;

/**
 * A single directory entry with metadata.
 * Mirrors the C cscandir_entry struct.
 */
public final class Entry {
    private final String path;
    private final boolean isSymlink;
    private final boolean isDir;
    private final boolean isFile;
    private final double ctime;
    private final double mtime;
    private final double atime;
    private final long size;
    private final boolean hasExt;
    private final int mode;
    private final long ino;
    private final long dev;
    private final long nlink;
    private final long blksize;
    private final long blocks;
    private final int uid;
    private final int gid;
    private final long rdev;

    // Package-private constructor for JNI
    Entry(String path, boolean isSymlink, boolean isDir, boolean isFile,
          double ctime, double mtime, double atime, long size,
          boolean hasExt, int mode, long ino, long dev, long nlink,
          long blksize, long blocks, int uid, int gid, long rdev) {
        this.path = path;
        this.isSymlink = isSymlink;
        this.isDir = isDir;
        this.isFile = isFile;
        this.ctime = ctime;
        this.mtime = mtime;
        this.atime = atime;
        this.size = size;
        this.hasExt = hasExt;
        this.mode = mode;
        this.ino = ino;
        this.dev = dev;
        this.nlink = nlink;
        this.blksize = blksize;
        this.blocks = blocks;
        this.uid = uid;
        this.gid = gid;
        this.rdev = rdev;
    }

    public String getPath() { return path; }
    public boolean isSymlink() { return isSymlink; }
    public boolean isDir() { return isDir; }
    public boolean isFile() { return isFile; }
    public double getCtime() { return ctime; }
    public double getMtime() { return mtime; }
    public double getAtime() { return atime; }
    public long getSize() { return size; }
    public boolean hasExt() { return hasExt; }
    public int getMode() { return mode; }
    public long getIno() { return ino; }
    public long getDev() { return dev; }
    public long getNlink() { return nlink; }
    public long getBlksize() { return blksize; }
    public long getBlocks() { return blocks; }
    public int getUid() { return uid; }
    public int getGid() { return gid; }
    public long getRdev() { return rdev; }

    @Override
    public String toString() {
        return String.format("Entry{path='%s', type=%s, size=%d, hasExt=%b}",
            path, isDir ? "DIR" : isFile ? "FILE" : isSymlink ? "SYMLINK" : "OTHER", size, hasExt);
    }
}