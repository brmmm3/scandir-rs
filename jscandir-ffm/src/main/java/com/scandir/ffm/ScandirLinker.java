package com.scandir.ffm;

import java.lang.foreign.AddressLayout;
import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.GroupLayout;
import java.lang.foreign.Linker;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.List;

/** FFM layouts and downcall handles for the exported cscandir C ABI. */
public final class ScandirLinker {
    private static final Linker LINKER = Linker.nativeLinker();
    private static final ValueLayout.OfInt C_INT = ValueLayout.JAVA_INT;
    private static final MemoryLayout C_SIZE_T = LINKER.canonicalLayouts().get("size_t");
    private static final ValueLayout.OfLong C_UINT64 = ValueLayout.JAVA_LONG;
    private static final ValueLayout.OfDouble C_DOUBLE = ValueLayout.JAVA_DOUBLE;
    private static final AddressLayout C_POINTER = ValueLayout.ADDRESS;
    private static final ValueLayout.OfByte C_BYTE = ValueLayout.JAVA_BYTE;

    public static final GroupLayout CSCANDIR_OPTIONS_LAYOUT = struct(
            C_BYTE.withName("sorted"), C_BYTE.withName("skip_hidden"),
            C_SIZE_T.withName("max_depth"), C_SIZE_T.withName("max_file_cnt"),
            C_POINTER.withName("dir_include"), C_SIZE_T.withName("dir_include_len"),
            C_POINTER.withName("dir_exclude"), C_SIZE_T.withName("dir_exclude_len"),
            C_POINTER.withName("file_include"), C_SIZE_T.withName("file_include_len"),
            C_POINTER.withName("file_exclude"), C_SIZE_T.withName("file_exclude_len"),
            C_BYTE.withName("case_sensitive"), C_BYTE.withName("follow_links"),
            C_INT.withName("return_type"));

    public static final GroupLayout CSCANDIR_ENTRY_LAYOUT = struct(
            C_POINTER.withName("path"), C_BYTE.withName("is_symlink"),
            C_BYTE.withName("is_dir"), C_BYTE.withName("is_file"),
            C_DOUBLE.withName("ctime"), C_DOUBLE.withName("mtime"),
            C_DOUBLE.withName("atime"), C_UINT64.withName("size"),
            C_BYTE.withName("has_ext"), C_INT.withName("mode"),
            C_UINT64.withName("ino"), C_UINT64.withName("dev"),
            C_UINT64.withName("nlink"), C_UINT64.withName("blksize"),
            C_UINT64.withName("blocks"), C_INT.withName("uid"), C_INT.withName("gid"),
            C_UINT64.withName("rdev"));

    public static final GroupLayout CSCANDIR_ENTRY_LIST_LAYOUT = struct(
            C_POINTER.withName("entries"), C_SIZE_T.withName("len"),
            C_SIZE_T.withName("capacity"));
    public static final GroupLayout CSCANDIR_STRING_LIST_LAYOUT = struct(
            C_POINTER.withName("items"), C_SIZE_T.withName("len"),
            C_SIZE_T.withName("capacity"));
    public static final GroupLayout CSCANDIR_ERROR_LAYOUT = struct(
            C_INT.withName("code"), C_POINTER.withName("message"));
    public static final GroupLayout CSCANDIR_STATISTICS_LAYOUT = struct(
            C_INT.withName("dirs"), C_INT.withName("files"), C_INT.withName("slinks"),
            C_INT.withName("hlinks"), C_INT.withName("devices"), C_INT.withName("pipes"),
            C_UINT64.withName("size"), C_UINT64.withName("usage"),
            C_DOUBLE.withName("duration"));

    public static final MethodHandle OPTIONS_INIT;
    public static final MethodHandle COLLECT;
    public static final MethodHandle COUNT;
    public static final MethodHandle FREE_ENTRY_LIST;
    public static final MethodHandle FREE_STRING_LIST;
    public static final MethodHandle FREE_ERROR;

    static {
        try {
            SymbolLookup lookup = lookupLibrary();
            OPTIONS_INIT = LINKER.downcallHandle(lookup.find("cscandir_options_init").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));
            COLLECT = LINKER.downcallHandle(lookup.find("cscandir_collect").orElseThrow(),
                    FunctionDescriptor.of(C_INT, C_POINTER, C_POINTER, C_POINTER, C_POINTER, C_POINTER));
            COUNT = LINKER.downcallHandle(lookup.find("cscandir_count").orElseThrow(),
                    FunctionDescriptor.of(C_INT, C_POINTER, C_POINTER, C_POINTER, C_POINTER, C_POINTER));
            FREE_ENTRY_LIST = LINKER.downcallHandle(lookup.find("cscandir_free_entry_list").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));
            FREE_STRING_LIST = LINKER.downcallHandle(lookup.find("cscandir_free_string_list").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));
            FREE_ERROR = LINKER.downcallHandle(lookup.find("cscandir_free_error").orElseThrow(),
                    FunctionDescriptor.ofVoid(C_POINTER));
        } catch (Throwable error) {
            throw new ExceptionInInitializerError(error);
        }
    }

    private ScandirLinker() { }

    private static GroupLayout struct(MemoryLayout... fields) {
        List<MemoryLayout> aligned = new ArrayList<>(fields.length * 2 + 1);
        long offset = 0;
        long alignment = 1;
        for (MemoryLayout field : fields) {
            long fieldAlignment = field.byteAlignment();
            long padding = (fieldAlignment - (offset % fieldAlignment)) % fieldAlignment;
            if (padding != 0) {
                aligned.add(MemoryLayout.paddingLayout(padding));
                offset += padding;
            }
            aligned.add(field);
            offset += field.byteSize();
            alignment = Math.max(alignment, fieldAlignment);
        }
        long padding = (alignment - (offset % alignment)) % alignment;
        if (padding != 0) aligned.add(MemoryLayout.paddingLayout(padding));
        return MemoryLayout.structLayout(aligned.toArray(MemoryLayout[]::new)).withByteAlignment(alignment);
    }

    private static SymbolLookup lookupLibrary() {
        String libraryName = System.mapLibraryName("cscandir");
        String configured = System.getProperty("cscandir.lib.path");
        if (configured != null && !configured.isBlank()) {
            Path path = Path.of(configured);
            if (Files.isDirectory(path)) path = path.resolve(libraryName);
            return SymbolLookup.libraryLookup(path.toAbsolutePath().toString(), Arena.global());
        }

        List<Path> paths = List.of(Path.of("../../target/release", libraryName),
                Path.of("../target/release", libraryName), Path.of("target/release", libraryName),
                Path.of("target/debug", libraryName));
        for (Path path : paths) {
            if (Files.isRegularFile(path)) {
                return SymbolLookup.libraryLookup(path.toAbsolutePath().toString(), Arena.global());
            }
        }
        String resource = "/native/" + platformKey() + "/" + libraryName;
        try (InputStream bundled = ScandirLinker.class.getResourceAsStream(resource)) {
            if (bundled != null) {
                Path extracted = Files.createTempFile("scandir-", libraryName);
                extracted.toFile().deleteOnExit();
                Files.copy(bundled, extracted, StandardCopyOption.REPLACE_EXISTING);
                return SymbolLookup.libraryLookup(extracted.toAbsolutePath().toString(), Arena.global());
            }
        } catch (IOException exception) {
            throw new IllegalStateException("Unable to load bundled native library " + resource, exception);
        }
        System.loadLibrary("cscandir");
        return SymbolLookup.loaderLookup();
    }

    private static String platformKey() {
        String os = System.getProperty("os.name", "").toLowerCase();
        String arch = System.getProperty("os.arch", "").toLowerCase();
        String platform = os.contains("win") ? "windows" : os.contains("mac") || os.contains("darwin")
                ? "macos" : "linux";
        String architecture = arch.equals("aarch64") || arch.equals("arm64") ? "arm64" : "x64";
        return platform + "-" + architecture;
    }

    public static long offset(MemoryLayout layout, String field) {
        return layout.byteOffset(MemoryLayout.PathElement.groupElement(field));
    }

    public static void setSizeT(MemorySegment segment, long offset, long value) {
        if (C_SIZE_T.byteSize() == Long.BYTES) segment.set(ValueLayout.JAVA_LONG, offset, value);
        else if (C_SIZE_T.byteSize() == Integer.BYTES && value <= 0xffff_ffffL) {
            segment.set(ValueLayout.JAVA_INT, offset, (int) value);
        } else throw new IllegalArgumentException("value does not fit native size_t: " + value);
    }

    public static long getSizeT(MemorySegment segment, long offset) {
        return C_SIZE_T.byteSize() == Long.BYTES
                ? segment.get(ValueLayout.JAVA_LONG, offset)
                : Integer.toUnsignedLong(segment.get(ValueLayout.JAVA_INT, offset));
    }

    /** Reads a native C string returned by cscandir while its allocation is alive. */
    public static String readCString(MemorySegment pointer) {
        return pointer.address() == 0 ? null : pointer.reinterpret(1L << 20).getString(0);
    }
}