package com.scandir;

import com.scandir.options.Options;
import com.scandir.result.CollectResult;
import com.scandir.result.Statistics;
import com.scandir.result.WalkResult;
import com.scandir.error.JScandirError;
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
        Files.writeString(root.resolve("café-雪.txt"), "unicode");
        return root;
    }

    @Test
    void collectReturnsEntriesAndStatistics() throws Exception {
        Path root = createTestTree();
        Options opts = Options.defaultOptions()
                .sorted(true)
                .skipHidden(true)
                .maxDepth(10)
                .fileInclude("*.txt");
        CollectResult result = JScandir.collect(root, opts);
        assertEquals(8, result.getEntries().size()); // root + five files + two descendant directories
        assertEquals(5, result.getStatistics().getFiles());
        assertEquals(2, result.getStatistics().getDirs());
        assertTrue(result.getErrors().isEmpty());
        assertTrue(result.getEntries().stream().anyMatch(entry -> entry.getPath().equals("test_tree")));
        assertTrue(result.getEntries().stream().anyMatch(entry -> entry.getPath().equals("sub/deep/d.txt")));
        assertTrue(result.getEntries().stream().anyMatch(entry -> entry.getPath().equals("café-雪.txt")));
        assertTrue(result.getStatistics().getDuration() >= 0.0);
    }

    @Test
    void countAndWalkUseSupportedAbi() throws Exception {
        Path root = createTestTree();
        Statistics stats = JScandir.count(root, Options.defaultOptions());
        assertEquals(5, stats.getFiles());
        assertEquals(2, stats.getDirs());
        assertEquals(2, JScandir.count(root.toRealPath().toString(), Options.defaultOptions()).getDirs());
        WalkResult walk = JScandir.walk(root, Options.defaultOptions());
        assertEquals(5, walk.getToc().getFiles().size());
        assertTrue(walk.getRoots().stream().anyMatch(entry -> entry.getDir().equals("sub/deep")));
    }

    @Test
    void nativeErrorIsPropagatedAndOutputsAreFreed() {
        JScandirError error = assertThrows(JScandirError.class,
                () -> JScandir.collect(tempDir.resolve("missing").toString(), null));
        assertEquals(3, error.getCode());
        assertNotNull(error.getMessage());
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