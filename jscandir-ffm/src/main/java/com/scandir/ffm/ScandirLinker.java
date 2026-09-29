package com.scandir.ffm;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Optional;

/**
 * FFM linker for the cscandir C library.
 * Provides Java method handles for all native cscandir functions.
 */
public final class ScandirLinker {
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LIB_LOOKUP;

    // C types
    private static final MemoryLayout C_INT = ValueLayout.JAVA_INT;
    private static final MemoryLayout C_SIZE_T = ValueLayout.JAVA_LONG;
    private static final MemoryLayout C_CHAR = ValueLayout.JAVA_BYTE;
    private static final MemoryLayout C_DOUBLE = ValueLayout.JAVA_DOUBLE;
    private static final MemoryLayout C_POINTER = ValueLayout.ADDRESS;

    // Struct layouts
    public static final GroupLayout CSCANDIR_OPTIONS_LAYOUT = MemoryLayout.structLayout(
            ValueLayout.JAVA_BYTE.withName("sorted"),
            ValueLayout.JAVA_BYTE.withName("skip_hidden"),
            C_SIZE_T.withName("max_depth"),
            C_SIZE_T.withName("max_file_cnt"),
            C_POINTER.withName("dir_include"),
            C_SIZE_T.withName("dir_include_len"),
            C_POINTER.withName("dir_exclude"),
            C_SIZE_T.withName("dir_exclude_len"),
            C_POINTER.withName("file_include"),
            C_SIZE_T.withName("file_include_len"),
            C_POINTER.withName("file_exclude"),
            C_SIZE_T.withName("file_exclude_len"),
            ValueLayout.JAVA_BYTE.withName("case_sensitive"),
            ValueLayout.JAVA_BYTE.withName("follow_links"),
            ValueLayout.JAVA_INT.withName("return_type")).withByteAlignment(8);

    public static final GroupLayout CSCANDIR_ENTRY_LAYOUT = MemoryLayout.structLayout(
            C_POINTER.withName("path"),
            ValueLayout.JAVA_BYTE.withName("is_symlink"),
            ValueLayout.JAVA_BYTE.withName("is_dir"),
            ValueLayout.JAVA_BYTE.withName("is_file"),
            C_DOUBLE.withName("ctime"),
            C_DOUBLE.withName("mtime"),
            C_DOUBLE.withName("atime"),
            ValueLayout.JAVA_LONG.withName("size"),
            ValueLayout.JAVA_BYTE.withName("has_ext"),
            ValueLayout.JAVA_INT.withName("mode"),
            ValueLayout.JAVA_LONG.withName("ino"),
            ValueLayout.JAVA_LONG.withName("dev"),
            ValueLayout.JAVA_LONG.withName("nlink"),
            ValueLayout.JAVA_LONG.withName("blksize"),
            ValueLayout.JAVA_LONG.withName("blocks"),
            ValueLayout.JAVA_INT.withName("uid"),
            ValueLayout.JAVA_INT.withName("gid"),
            ValueLayout.JAVA_LONG.withName("rdev")).withByteAlignment(8);

    public static final GroupLayout CSCANDIR_ENTRY_LIST_LAYOUT = MemoryLayout.structLayout(
            C_POINTER.withName("entries"),
            C_SIZE_T.withName("len"),
            C_SIZE_T.withName("capacity")).withByteAlignment(8);

    public static final GroupLayout CSCANDIR_STRING_LIST_LAYOUT = MemoryLayout.structLayout(
            C_POINTER.withName("items"),
            C_SIZE_T.withName("len"),
            C_SIZE_T.withName("capacity")).withByteAlignment(8);

    public static final GroupLayout CSCANDIR_TOC_LAYOUT = MemoryLayout.structLayout(
            CSCANDIR_STRING_LIST_LAYOUT.withName("dirs"),
            CSCANDIR_STRING_LIST_LAYOUT.withName("files"),
            CSCANDIR_STRING_LIST_LAYOUT.withName("symlinks"),
            CSCANDIR_STRING_LIST_LAYOUT.withName("other"),
            CSCANDIR_STRING_LIST_LAYOUT.withName("errors")).withByteAlignment(8);

    public static final GroupLayout CSCANDIR_WALK_ENTRY_LAYOUT = MemoryLayout.structLayout(
            C_POINTER.withName("path"),
            CSCANDIR_TOC_LAYOUT.withName("toc")).withByteAlignment(8);

    public static final GroupLayout CSCANDIR_WALK_ENTRY_LIST_LAYOUT = MemoryLayout.structLayout(
            C_POINTER.withName("entries"),
            C_SIZE_T.withName("len"),
            C_SIZE_T.withName("capacity")).withByteAlignment(8);

    public static final GroupLayout CSCANDIR_ERROR_LAYOUT = MemoryLayout.structLayout(
            ValueLayout.JAVA_INT.withName("code"),
            C_POINTER.withName("message")).withByteAlignment(8);

    public static final GroupLayout CSCANDIR_STATISTICS_LAYOUT = MemoryLayout.structLayout(
            ValueLayout.JAVA_INT.withName("dirs"),
            ValueLayout.JAVA_INT.withName("files"),
            ValueLayout.JAVA_INT.withName("slinks"),
            ValueLayout.JAVA_INT.withName("hlinks"),
            ValueLayout.JAVA_INT.withName("devices"),
            ValueLayout.JAVA_INT.withName("pipes"),
            ValueLayout.JAVA_LONG.withName("size"),
            ValueLayout.JAVA_LONG.withName("usage"),
            ValueLayout.JAVA_DOUBLE.withName("duration")).withByteAlignment(8);

    static {
        // Try to find the library in standard locations
        String libName = System.mapLibraryName("cscandir");
        String libPath = System.getProperty("cscandir.lib.path");

        SymbolLookup lookup;
        if (libPath != null) {
            lookup = SymbolLookup.libraryLookup(libPath, Arena.global());
        } else {
            // Try to find in common locations
            List<String> searchPaths = List.of(
                    "../../target/release",
                    "../target/release",
                    "target/release",
                    "target/debug",
                    "/usr/local/lib",
                    "/usr/lib");

            SymbolLookup fallback = SymbolLookup.loaderLookup();
            for (String path : searchPaths) {
                try {
                    Path p = Paths.get(path).resolve("libcscandir"
                            + (System.getProperty("os.name").toLowerCase().contains("win") ? ".dll" : ".so"));
                    if (p.toFile().exists()) {
                        fallback = SymbolLookup.libraryLookup(p.toAbsolutePath().toString(), Arena.global());
                        break;
                    }
                } catch (Exception ignored) {
                }
            }
            lookup = fallback;
        }

        LIB_LOOKUP = lookup;
    }

    private ScandirLinker() {
    }

    /**
     * Returns the default symbol lookup for the native cscandir library.
     */
    public static SymbolLookup defaultLookup() {
        return LIB_LOOKUP;
    }

    /**
     * Allocates a zeroed memory segment for the given layout within the arena.
     */
    public static MemorySegment allocateMemory(Arena arena, MemoryLayout layout) {
        return arena.allocate(layout);
    }

    /**
     * Allocates a zeroed memory segment for {@code count} elements of the given
     * layout within the arena.
     */
    public static MemorySegment allocateMemory(Arena arena, MemoryLayout layout, long count) {
        return arena.allocate(layout, count);
    }

    /**
     * Allocates a native string (NUL-terminated) within the arena.
     */
    public static MemorySegment allocateString(Arena arena, String str) {
        return arena.allocateFrom(str);
    }

    /**
     * Reads a NUL-terminated native string from the given segment.
     */
    public static String toString(MemorySegment segment) {
        return segment.getString(0);
    }

    // Native function method handles
    public static final MethodHandle CSCANDIR_OPTIONS_INIT;
    public static final MethodHandle CSCANDIR_COLLECT;
    public static final MethodHandle CSCANDIR_COUNT;
    public static final MethodHandle CSCANDIR_WALK;
    public static final MethodHandle CSCANDIR_FREE_ENTRY_LIST;
    public static final MethodHandle CSCANDIR_FREE_STRING_LIST;
    public static final MethodHandle CSCANDIR_FREE_TOC;
    public static final MethodHandle CSCANDIR_FREE_WALK_ENTRY_LIST;
    public static final MethodHandle CSCANDIR_FREE_ERROR;
    public static final MethodHandle CSCANDIR_ENTRY_LIST_INIT;
    public static final MethodHandle CSCANDIR_STRING_LIST_INIT;
    public static final MethodHandle CSCANDIR_TOC_INIT;
    public static final MethodHandle CSCANDIR_WALK_ENTRY_LIST_INIT;
    public static final MethodHandle CSCANDIR_ERROR_INIT;

    static {
        try {
            // Load the native library
            System.loadLibrary("cscandir");

            // Lookup function symbols
            var lookup = LIB_LOOKUP;

            CSCANDIR_OPTIONS_INIT = LINKER.downcallHandle(
                    lookup.find("cscandir_options_init").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_COLLECT = LINKER.downcallHandle(
                    lookup.find("cscandir_collect").orElseThrow(),
                    FunctionDescriptor.of(C_INT,
                            C_POINTER, // root_path
                            C_POINTER, // options
                            C_POINTER, // out_entries
                            C_POINTER, // out_errors
                            C_POINTER, // out_stats
                            C_POINTER // out_error
                    ));

            CSCANDIR_COUNT = LINKER.downcallHandle(
                    lookup.find("cscandir_count").orElseThrow(),
                    FunctionDescriptor.of(C_INT,
                            C_POINTER, // root_path
                            C_POINTER, // options
                            C_POINTER, // out_stats
                            C_POINTER, // out_errors
                            C_POINTER // out_error
                    ));

            CSCANDIR_WALK = LINKER.downcallHandle(
                    lookup.find("cscandir_walk").orElseThrow(),
                    FunctionDescriptor.of(C_INT,
                            C_POINTER, // root_path
                            C_POINTER, // options
                            C_POINTER, // out_toc
                            C_POINTER, // out_entries
                            C_POINTER, // out_stats
                            C_POINTER // out_error
                    ));

            CSCANDIR_FREE_ENTRY_LIST = LINKER.downcallHandle(
                    lookup.find("cscandir_free_entry_list").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_FREE_STRING_LIST = LINKER.downcallHandle(
                    lookup.find("cscandir_free_string_list").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_FREE_TOC = LINKER.downcallHandle(
                    lookup.find("cscandir_free_toc").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_FREE_WALK_ENTRY_LIST = LINKER.downcallHandle(
                    lookup.find("cscandir_free_walk_entry_list").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_FREE_ERROR = LINKER.downcallHandle(
                    lookup.find("cscandir_free_error").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_ENTRY_LIST_INIT = LINKER.downcallHandle(
                    lookup.find("cscandir_entry_list_init").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_STRING_LIST_INIT = LINKER.downcallHandle(
                    lookup.find("cscandir_string_list_init").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_TOC_INIT = LINKER.downcallHandle(
                    lookup.find("cscandir_toc_init").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_WALK_ENTRY_LIST_INIT = LINKER.downcallHandle(
                    lookup.find("cscandir_walk_entry_list_init").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

            CSCANDIR_ERROR_INIT = LINKER.downcallHandle(
                    lookup.find("cscandir_error_init").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));

        } catch (Exception e) {
            throw new ExceptionInInitializerError("Failed to initialize ScandirLinker: " + e.getMessage());
        }
    }
}