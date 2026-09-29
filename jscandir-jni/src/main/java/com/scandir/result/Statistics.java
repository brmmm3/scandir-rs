package com.scandir.result;

/**
 * Statistics collected during a scan.
 * Mirrors the C cscandir_statistics struct.
 */
public final class Statistics {
    private final int dirs;
    private final int files;
    private final int slinks;
    private final int hlinks;
    private final int devices;
    private final int pipes;
    private final long size;
    private final long usage;
    private final double duration;

    public Statistics(int dirs, int files, int slinks, int hlinks,
            int devices, int pipes, long size, long usage, double duration) {
        this.dirs = dirs;
        this.files = files;
        this.slinks = slinks;
        this.hlinks = hlinks;
        this.devices = devices;
        this.pipes = pipes;
        this.size = size;
        this.usage = usage;
        this.duration = duration;
    }

    public int getDirs() {
        return dirs;
    }

    public int getFiles() {
        return files;
    }

    public int getSlinks() {
        return slinks;
    }

    public int getHlinks() {
        return hlinks;
    }

    public int getDevices() {
        return devices;
    }

    public int getPipes() {
        return pipes;
    }

    public long getSize() {
        return size;
    }

    public long getUsage() {
        return usage;
    }

    public double getDuration() {
        return duration;
    }

    @Override
    public String toString() {
        return String.format("Statistics{dirs=%d, files=%d, slinks=%d, hlinks=%d, " +
                "devices=%d, pipes=%d, size=%d, usage=%d, duration=%.4f}",
                dirs, files, slinks, hlinks, devices, pipes, size, usage, duration);
    }
}