package com.scandir.result;

import java.util.List;

/**
 * Result of a walk operation.
 */
public final class WalkResult {
    private final Toc toc;
    private final List<WalkEntry> roots;
    private final Statistics statistics;

    public WalkResult(Toc toc, List<WalkEntry> roots, Statistics statistics) {
        this.toc = toc;
        this.roots = List.copyOf(roots);
        this.statistics = statistics;
    }

    public Toc getToc() { return toc; }
    public List<WalkEntry> getRoots() { return roots; }
    public Statistics getStatistics() { return statistics; }

    @Override
    public String toString() {
        return String.format("WalkResult{toc=%s, roots=%d, stats=%s}",
            toc, roots.size(), statistics);
    }
}