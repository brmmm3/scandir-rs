package com.scandir;

import com.scandir.options.Options;
import com.scandir.options.ReturnType;
import com.scandir.result.CollectResult;
import com.scandir.result.Statistics;
import com.scandir.result.WalkResult;
import com.scandir.error.JScandirException;
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

    @Test
    void nativeCollectCountAndWalk() throws Exception {
        Path root = createTestTree();
        CollectResult result = com.scandir.jni.JScandir.collect(root, Options.defaultOptions()
                .sorted(true).returnType(ReturnType.EXT));
        assertEquals(8, result.getEntries().size()); // root + five files + two descendant directories
        assertEquals(5, result.getStatistics().getFiles());
        assertEquals(2, result.getStatistics().getDirs());
        assertTrue(result.getEntries().stream().anyMatch(entry -> entry.getPath().equals("sub/deep/d.txt")));
        assertTrue(result.getEntries().stream().anyMatch(entry -> entry.getPath().equals("café-雪.txt")));
        assertTrue(result.getEntries().stream().filter(entry -> entry.isFile()).allMatch(entry -> entry.hasExt()));

        Statistics stats = com.scandir.jni.JScandir.count(root, Options.defaultOptions());
        assertEquals(5, stats.getFiles());
        assertEquals(2, stats.getDirs());
        assertEquals(2, com.scandir.jni.JScandir.collect(root.toString(), Options.defaultOptions())
                .getStatistics().getDirs());

        WalkResult walk = com.scandir.jni.JScandir.walk(root, Options.defaultOptions());
        assertEquals(5, walk.getToc().getFiles().size());
        assertTrue(walk.getRoots().stream().anyMatch(entry -> entry.getDir().equals("sub/deep")));
    }

    @Test
    void invalidRootIsReported() {
        JScandirException error = assertThrows(JScandirException.class,
                () -> com.scandir.jni.JScandir.collect(tempDir.resolve("missing").toString(), null));
        assertEquals(3, error.getCode());
    }
}