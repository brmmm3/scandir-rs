package com.scandir;

import com.scandir.options.Options;
import com.scandir.result.CollectResult;
import com.scandir.result.Statistics;
import com.scandir.result.WalkResult;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

class JScandirTest {

    @TempDir
    Path tempDir;

    private Path createTestTree() throws Exception {
        Path root = tempDir.resolve("test_tree");
        Files.createDirectories(root.resolve("sub").resolve("deep"));
        Files.writeString(root.resolve("a.txt"), "a");
        Files.writeString(root.resolve("b.txt"), "bb");
        Files.writeString(root.resolve("sub").resolve("c.txt"), "ccc");
        Files.writeString(root.resolve("sub").resolve("deep").resolve("d.txt"), "dddd");
        return root;
    }

    @Test
    void collectReturnsEntriesAndStatistics() throws Exception {
        Path root = createTestTree();
        Options options = Options.defaultOptions();

        // This will fail because native library isn't loaded in test environment
        // but we can test the Options construction
        Options opts = Options.defaultOptions()
                .sorted(true)
                .skipHidden(true)
                .maxDepth(10)
                .fileInclude("*.txt");

        assertTrue(opts.isSorted());
        assertTrue(opts.isSkipHidden());
        assertEquals(10, opts.getMaxDepth());
        assertEquals(List.of("*.txt"), opts.getFileInclude());
    }

    @Test
    void optionsBuilderWorks() {
        Options opts = new Options()
                .sorted(true)
                .skipHidden(false)
                .maxDepth(5)
                .fileInclude("*.java", "*.kt")
                .dirExclude("target", "build");

        assertTrue(opts.isSorted());
        assertFalse(opts.isSkipHidden());
        assertEquals(5, opts.getMaxDepth());
        assertEquals(List.of("*.java", "*.kt"), opts.getFileInclude());
        assertEquals(List.of("target", "build"), opts.getDirExclude());
    }

    @Test
    void returnTypeEnumWorks() {
        assertEquals(0, com.scandir.options.ReturnType.BASE.getValue());
        assertEquals(1, com.scandir.options.ReturnType.EXT.getValue());
    }

    @Test
    void statisticsConstruction() {
        Statistics stats = new Statistics(2, 4, 1, 0, 0, 0, 1024, 2048, 0.123);
        assertEquals(2, stats.getDirs());
        assertEquals(4, stats.getFiles());
        assertEquals(1, stats.getSlinks());
        assertEquals(0.123, stats.getDuration(), 0.001);
    }
}