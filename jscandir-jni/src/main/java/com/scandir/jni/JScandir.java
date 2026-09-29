package com.scandir.jni;

import com.scandir.options.Options;
import com.scandir.options.ReturnType;
import com.scandir.result.*;
import com.scandir.error.JScandirException;

import java.nio.file.Path;

/**
 * Main entry point for the jscandir-jni library.
 * Provides high-level API for directory scanning using the cscandir C library
 * via Java Native Interface (JNI).
 */
public final class JScandir {

    static {
        // Load the native library
        String libName = System.mapLibraryName("jscandir");
        String libPath = System.getProperty("jscandir.lib.path");
        if (libPath != null) {
            System.load(libPath + "/" + libName);
        } else {
            // Try standard locations
            String[] paths = {
                "../../target/release",
                "../target/release",
                "target/release",
                "target/debug",
                "/usr/local/lib",
                "/usr/lib"
            };
            String libName = System.mapLibraryName("jscandir");
            boolean loaded = false;
            for (String path : paths) {
                try {
                    java.nio.file.Path p = java.nio.file.Paths.get(path, libName);
                    if (p.toFile().exists()) {
                        System.load(p.toAbsolutePath().toString());
                        loaded = true;
                        break;
                    }
                } catch (Exception ignored) {}
            }
            if (!loaded) {
                System.loadLibrary("jscandir");
            }
        }
    }

    private JScandir() {}

    /**
     * Collects directory entries with full metadata.
     * 
     * @param root The root directory path to scan
     * @param options Scan options (null for defaults)
     * @return CollectResult containing entries, errors, and statistics
     * @throws JScandirException if the scan fails
     */
    public static native CollectResult collect(String root, Options options) throws JScandirException;

    /**
     * Collects directory entries with full metadata.
     * 
     * @param root The root directory path to scan
     * @param options Scan options (null for defaults)
     * @return CollectResult containing entries, errors, and statistics
     * @throws JScandirException if the scan fails
     */
    public static CollectResult collect(Path root, Options options) {
        return collect(root.toString(), options);
    }

    /**
     * Collects aggregate directory statistics without materializing entries.
     * 
     * @param root The root directory path to scan
     * @param options Scan options (null for defaults)
     * @return Statistics object
     * @throws JScandirException if the scan fails
     */
    public static native Statistics count(String root, Options options) throws JScandirException;

    public static Statistics count(Path root, Options options) {
        return count(root.toString(), options);
    }

    /**
     * Collects directory entries grouped by directory (Walk).
     * 
     * @param root The root directory path to scan
     * @param options Scan options (null for defaults)
     * @return WalkResult with per-directory breakdown
     * @throws JScandirException if the scan fails
     */
    public static native WalkResult walk(String root, Options options) throws JScandirException;

    public static WalkResult walk(Path root, Options options) {
        return walk(root.toString(), options);
    }

    private JScandir() {}
}