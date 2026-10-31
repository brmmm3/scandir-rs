package com.scandir.jni;

import com.scandir.error.JScandirException;
import com.scandir.options.Options;
import com.scandir.result.CollectResult;
import com.scandir.result.Statistics;
import com.scandir.result.Toc;
import com.scandir.result.WalkEntry;
import com.scandir.result.WalkResult;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.TreeMap;

/** Java API over cscandir using a JNI bridge. */
public final class JScandir {
    static {
        String library = System.mapLibraryName("jscandir");
        String configured = System.getProperty("jscandir.lib.path");
        Path configuredPath = configured == null || configured.isBlank() ? null : Path.of(configured);
        Path builtLibrary = Path.of("target", "native", library);
        if (configuredPath != null && java.nio.file.Files.isDirectory(configuredPath)) {
            configuredPath = configuredPath.resolve(library);
        }
        if (configuredPath != null && java.nio.file.Files.isRegularFile(configuredPath)) {
            System.load(configuredPath.toAbsolutePath().toString());
        } else if (java.nio.file.Files.isRegularFile(builtLibrary)) {
            System.load(builtLibrary.toAbsolutePath().toString());
        } else if (!loadBundledLibraries(library)) {
            System.loadLibrary("jscandir");
        }
    }

    private static boolean loadBundledLibraries(String bridgeName) {
        String platform = platformKey();
        Path extractionDirectory;
        try {
            extractionDirectory = Files.createTempDirectory("scandir-" + platform + "-");
            extractionDirectory.toFile().deleteOnExit();
            for (String library : List.of(System.mapLibraryName("cscandir"), bridgeName)) {
                String resource = "/native/" + platform + "/" + library;
                try (InputStream input = JScandir.class.getResourceAsStream(resource)) {
                    if (input == null) return false;
                    Path extracted = extractionDirectory.resolve(library);
                    Files.copy(input, extracted, StandardCopyOption.REPLACE_EXISTING);
                    extracted.toFile().deleteOnExit();
                }
            }
            Path nativeDirectory = extractionDirectory;
            System.load(nativeDirectory.resolve(System.mapLibraryName("cscandir")).toAbsolutePath().toString());
            System.load(nativeDirectory.resolve(bridgeName).toAbsolutePath().toString());
            return true;
        } catch (IOException exception) {
            throw new ExceptionInInitializerError(exception);
        }
    }

    private static String platformKey() {
        String os = System.getProperty("os.name", "").toLowerCase();
        String arch = System.getProperty("os.arch", "").toLowerCase();
        String platform = os.contains("win") ? "windows" : os.contains("mac") || os.contains("darwin")
                ? "macos" : "linux";
        String architecture = arch.equals("aarch64") || arch.equals("arm64") ? "arm64" : "x64";
        return platform + "-" + architecture;
    }

    private JScandir() { }

    public static native CollectResult collect(String root, Options options) throws JScandirException;
    public static native Statistics count(String root, Options options) throws JScandirException;

    public static CollectResult collect(Path root, Options options) throws JScandirException {
        return collect(Objects.requireNonNull(root, "root").toString(), options);
    }

    public static Statistics count(Path root, Options options) throws JScandirException {
        return count(Objects.requireNonNull(root, "root").toString(), options);
    }

    /** Groups a single collect result by parent directory; cscandir has no walk ABI. */
    public static WalkResult walk(String root, Options options) throws JScandirException {
        long started = System.nanoTime();
        CollectResult collected = collect(root, options);
        Map<String, TocParts> grouped = new TreeMap<>();
        TocParts merged = new TocParts();
        List<com.scandir.result.Entry> entries = collected.getEntries();
        int firstCounted = hasSyntheticRoot(root, entries) ? 1 : 0;
        for (int i = firstCounted; i < entries.size(); i++) {
            var entry = entries.get(i);
            String path = entry.getPath();
            int separator = path.lastIndexOf('/');
            String parent = separator < 0 ? "" : path.substring(0, separator);
            String name = separator < 0 ? path : path.substring(separator + 1);
            TocParts part = grouped.computeIfAbsent(parent, ignored -> new TocParts());
            if (entry.isDir()) { part.dirs.add(name); merged.dirs.add(path); }
            else if (entry.isFile()) { part.files.add(name); merged.files.add(path); }
            else if (entry.isSymlink()) { part.symlinks.add(name); merged.symlinks.add(path); }
            else { part.other.add(name); merged.other.add(path); }
        }
        merged.errors.addAll(collected.getErrors());
        List<WalkEntry> roots = new ArrayList<>(grouped.size());
        grouped.forEach((dir, part) -> roots.add(WalkEntry.fromJni(dir, part.toToc())));
        Statistics stats = collected.getStatistics();
        Statistics resultStats = new Statistics(stats.getDirs(), stats.getFiles(), stats.getSlinks(),
                stats.getHlinks(), stats.getDevices(), stats.getPipes(), stats.getSize(), stats.getUsage(),
                (System.nanoTime() - started) / 1_000_000_000.0);
        return WalkResult.fromJni(merged.toToc(), roots, resultStats);
    }

    public static WalkResult walk(Path root, Options options) throws JScandirException {
        return walk(Objects.requireNonNull(root, "root").toString(), options);
    }

    private static final class TocParts {
        final List<String> dirs = new ArrayList<>(), files = new ArrayList<>();
        final List<String> symlinks = new ArrayList<>(), other = new ArrayList<>(), errors = new ArrayList<>();
        Toc toToc() { return Toc.fromJni(dirs, files, symlinks, other, errors); }
    }

    private static boolean hasSyntheticRoot(String root, List<com.scandir.result.Entry> entries) {
        if (entries.isEmpty()) return false;
        var first = entries.get(0);
        Path input = Path.of(root).normalize();
        Path normalized = input.isAbsolute() ? input : Path.of("").toAbsolutePath().resolve(input).normalize();
        try {
            normalized = normalized.toRealPath();
        } catch (IOException ignored) {
            // Keep the normalized absolute path for a root that cannot be resolved.
        }
        Path name = normalized.getFileName();
        return first.isDir() && name != null && first.getPath().equals(name.toString());
    }
}