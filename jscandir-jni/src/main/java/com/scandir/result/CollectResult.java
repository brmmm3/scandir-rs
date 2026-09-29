package com.scandir.result;

import java.util.List;

/**
 * Result of a collect operation.
 */
public final class CollectResult {
    private final java.util.List<Entry> entries;
    private final java.util.List<String> errors;
    private final Statistics statistics;

    // Package-private constructor for JNI
    CollectResult(java.util.List<Entry> entries, java.util.List<String> errors, Statistics statistics) {
        this.entries = List.copyOf(entries);
        this.errors = List.copyOf(errors);
        this.statistics = statistics;
    }

    public java.util.List<Entry> getEntries() { return entries; }
    public java.util.List<String> getErrors() { return errors; }
    public Statistics getStatistics() { return statistics; }

    @Override
    public String toString() {
        return String.format("CollectResult{entries=%d, errors=%d, stats=%s}",
            entries.size(), errors.size(), statistics);
    }
}