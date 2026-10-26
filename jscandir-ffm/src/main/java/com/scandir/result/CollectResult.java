package com.scandir.result;

import java.util.List;

/**
 * Result of a collect operation.
 */
public final class CollectResult {
    private final List<Entry> entries;
    private final List<String> errors;
    private final Statistics statistics;

    public CollectResult(List<Entry> entries, List<String> errors, Statistics statistics) {
        this.entries = List.copyOf(entries);
        this.errors = List.copyOf(errors);
        this.statistics = statistics;
    }

    public List<Entry> getEntries() { return entries; }
    public List<String> getErrors() { return errors; }
    public Statistics getStatistics() { return statistics; }

    @Override
    public String toString() {
        return String.format("CollectResult{entries=%d, errors=%d, stats=%s}",
            entries.size(), errors.size(), statistics);
    }
}