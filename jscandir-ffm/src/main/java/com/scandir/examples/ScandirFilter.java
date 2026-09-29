package com.scandir.examples;

import com.scandir.JScandir;
import com.scandir.options.Options;
import com.scandir.options.ReturnType;
import com.scandir.result.CollectResult;
import com.scandir.result.Entry;
import com.scandir.error.JScandirError;

/**
 * Example demonstrating filter options.
 * 
 * Usage: java -cp target/jscandir-ffm-2.10.1.jar:target/native/libcscandir.so \
 *   -Dcscandir.lib.path=/path/to/libcscandir.so \
 *   com.scandir.examples.ScandirFilter [directory]
 */
public class ScandirFilter {

    public static void main(String[] args) {
        String root = args.length > 0 ? args[0] : ".";

        // Filter for only .java and .kt files, exclude test directories
        Options options = Options.defaultOptions()
            .fileInclude("*.java", "*.kt")
            .dirExclude("target", "build", ".git", "node_modules")
            .sorted(true)
            .returnType(ReturnType.EXT);

        try {
            System.out.println("Filtering: " + root);
            System.out.println("  File patterns: *.java, *.kt");
            System.out.println("  Excluded dirs: target, build, .git, node_modules\n");

            CollectResult result = JScandir.collect(root, options);

            System.out.println("Found " + result.getEntries().size() + " entries");
            System.out.println("Directories: " + result.getStatistics().getDirs() +
                "  Files: " + result.getStatistics().getFiles());

            int fileCount = 0;
            for (Entry entry : result.getEntries()) {
                if (entry.isFile()) {
                    fileCount++;
                    if (fileCount <= 20) {
                        System.out.println("  " + entry.getPath());
                    } else if (fileCount == 21) {
                        System.out.println("  ... and " + (result.getEntries().size() - 20) + " more");
                    }
                }
            }
            System.out.println("Total matching files: " + fileCount);

        } catch (com.scandir.error.JScandirError e) {
            System.err.println("Error: " + e.getMessage() + " (code " + e.getCode() + ")");
            System.exit(1);
        }
    }
}