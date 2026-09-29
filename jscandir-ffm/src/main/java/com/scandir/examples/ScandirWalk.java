package com.scandir.examples;

import com.scandir.JScandir;
import com.scandir.options.Options;
import com.scandir.result.WalkResult;
import com.scandir.error.JScandirError;

import java.nio.file.Path;
import java.nio.file.Paths;

/**
 * Example demonstrating the Walk function.
 * 
 * Usage: java -cp target/jscandir-ffm-2.10.1.jar:target/native/libcscandir.so \
 *   -Dcscandir.lib.path=/path/to/libcscandir.so \
 *   com.scandir.examples.ScandirWalk [directory]
 */
public class ScandirWalk {

    public static void main(String[] args) {
        String root = args.length > 0 ? args[0] : ".";

        // Walk defaults SkipHidden to false. Set it to true to match
        // the Rust Walk default.
        Options options = Options.defaultOptions()
            .skipHidden(true)
            .sorted(true);

        try {
            System.out.println("Walking: " + root);
            com.scandir.result.WalkResult result = JScandir.walk(root, options);

            System.out.println("Merged TOC: dirs=" + result.getToc().getDirs().size() +
                " files=" + result.getToc().getFiles().size() +
                " symlinks=" + result.getToc().getSymlinks().size() +
                " other=" + result.getToc().getOther().size());
            System.out.println("Directories visited: " + result.getRoots().size());
            System.out.println("Statistics: dirs=" + result.getStatistics().getDirs() +
                " files=" + result.getStatistics().getFiles() +
                " duration=" + result.getStatistics().getDuration() + "s\n");

            for (com.scandir.result.WalkEntry entry : result.getRoots()) {
                String name = entry.getDir().isEmpty() ? "." : entry.getDir();
                System.out.printf("%s: dirs=%d files=%d symlinks=%d other=%d%n",
                    name,
                    entry.getToc().getDirs().size(),
                    entry.getToc().getFiles().size(),
                    entry.getToc().getSymlinks().size(),
                    entry.getToc().getOther().size());
            }

            if (!result.getToc().getErrors().isEmpty()) {
                System.out.println("\nScan errors:");
                for (String err : result.getToc().getErrors()) {
                    System.out.println("  " + err);
                }
            }

        } catch (com.scandir.error.JScandirError e) {
            System.err.println("Error: " + e.getMessage() + " (code " + e.getCode() + ")");
            System.exit(1);
        }
    }
}