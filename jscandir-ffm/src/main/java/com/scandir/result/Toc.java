package com.scandir.result;

import java.util.List;

/**
 * Table of contents - grouped entry names by type.
 * Mirrors the C cscandir_toc struct.
 */
public final class Toc {
    private final List<String> dirs;
    private final List<String> files;
    private final List<String> symlinks;
    private final List<String> other;
    private final List<String> errors;

    public Toc(List<String> dirs, List<String> files, List<String> symlinks,
               List<String> other, List<String> errors) {
        this.dirs = List.copyOf(dirs);
        this.files = List.copyOf(files);
        this.symlinks = List.copyOf(symlinks);
        this.other = List.copyOf(other);
        this.errors = List.copyOf(errors);
    }

    public List<String> getDirs() { return dirs; }
    public List<String> getFiles() { return files; }
    public List<String> getSymlinks() { return symlinks; }
    public List<String> getOther() { return other; }
    public List<String> getErrors() { return errors; }

    public boolean isEmpty() {
        return dirs.isEmpty() && files.isEmpty() && symlinks.isEmpty() &&
               other.isEmpty() && errors.isEmpty();
    }

    @Override
    public String toString() {
        return String.format("Toc{dirs=%d, files=%d, symlinks=%d, other=%d, errors=%d}",
            dirs.size(), files.size(), symlinks.size(), other.size(), errors.size());
    }
}