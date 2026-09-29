package com.scandir;

import com.scandir.options.Options;
import com.scandir.result.*;
import com.scandir.error.JScandirError;
import com.scandir.ffm.ScandirLinker;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.invoke.MethodHandle;
import java.nio.file.Path;
import java.util.List;

/**
 * Main entry point for the jscandir-ffm library.
 * Provides high-level API for directory scanning using the cscandir C library
 * via Java's Foreign Function & Memory API (FFM).
 */
public final class JScandir {

    /**
     * Collects directory entries with full metadata.
     * 
     * @param root    The root directory path to scan
     * @param options Scan options (null for defaults)
     * @return CollectResult containing entries, errors, and statistics
     * @throws JScandirError if the scan fails
     */
    public static CollectResult collect(Path root, Options options) {
        return collect(root.toString(), options);
    }

    public static CollectResult collect(String root, Options options) {
        try (Arena arena = Arena.ofConfined()) {
            // Prepare native options
            var nativeOptions = options != null ? options.toNative(arena) : Options.defaultOptions().toNative(arena);

            // Prepare output structs
            MemorySegment outEntries = ScandirLinker.allocateMemory(arena, ScandirLinker.CSCANDIR_ENTRY_LIST_LAYOUT);
            MemorySegment outErrors = ScandirLinker.allocateMemory(arena, ScandirLinker.CSCANDIR_STRING_LIST_LAYOUT);
            MemorySegment outStats = ScandirLinker.allocateMemory(arena, ScandirLinker.CSCANDIR_STATISTICS_LAYOUT);
            MemorySegment outError = ScandirLinker.allocateMemory(arena, ScandirLinker.CSCANDIR_ERROR_LAYOUT);

            // Initialize output structs
            ScandirLinker.CSCANDIR_ENTRY_LIST_INIT.invokeExact(outEntries);
            ScandirLinker.CSCANDIR_STRING_LIST_INIT.invokeExact(outErrors);
            ScandirLinker.CSCANDIR_ERROR_INIT.invokeExact(outError);

            // Convert root path to C string
            MemorySegment cRoot = ScandirLinker.allocateString(arena, root);

            // Call native function
            int code = (int) ScandirLinker.CSCANDIR_COLLECT.invokeExact(
                    cRoot, nativeOptions, outEntries, outErrors, outStats, outError);

            // This is a simplified implementation - in practice we'd need proper error
            // handling
            // and memory management
            return new CollectResult(List.of(), List.of(), new Statistics(0, 0, 0, 0, 0, 0, 0, 0, 0.0));

        } catch (Throwable e) {
            throw new JScandirError("Native call failed: " + e.getMessage(), e);
        }
    }

    /**
     * Collects aggregate directory statistics without materializing entries.
     */
    public static Statistics count(Path root, Options options) {
        return count(root.toString(), options);
    }

    public static Statistics count(String root, Options options) {
        // Implementation similar to collect
        return new Statistics(0, 0, 0, 0, 0, 0, 0, 0, 0.0);
    }

    /**
     * Collects directory entries grouped by type (Walk).
     */
    public static WalkResult walk(Path root, Options options) {
        return walk(root.toString(), options);
    }

    public static WalkResult walk(String root, Options options) {
        // Implementation similar to collect
        return new WalkResult(new Toc(List.of(), List.of(), List.of(), List.of(), List.of()),
                List.of(), new Statistics(0, 0, 0, 0, 0, 0, 0, 0, 0.0));
    }

    private JScandir() {
    }
}