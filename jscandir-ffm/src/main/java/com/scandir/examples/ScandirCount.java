package com.scandir.examples;

import com.scandir.JScandir;
import com.scandir.options.Options;
import com.scandir.result.Statistics;
import com.scandir.error.JScandirError;

/**
 * Example demonstrating the Count function for fast statistics.
 * 
 * Usage: java -cp target/jscandir-ffm-2.10.1.jar:target/native/libcscandir.so \
 *   -Dcscandir.lib.path=/path/to/libcscandir.so \
 *   com.scandir.examples.ScandirCount [directory]
 */
public class ScandirCount {

    public static void main(String[] args) {
        String root = args.length > 0 ? args[0] : ".";

        Options options = Options.defaultOptions()
            .skipHidden(true);

        try {
            System.out.println("Counting: " + root);
            Statistics stats = JScandir.count(root, options);

            System.out.println("Statistics:");
            System.out.println("  Directories: " + stats.getDirs());
            System.out.println("  Files: " + stats.getFiles());
            System.out.println("  Symlinks: " + stats.getSlinks());
            System.out.println("  Hardlinks: " + stats.getHlinks());
            System.out.println("  Devices: " + stats.getDevices());
            System.out.println("  Pipes: " + stats.getPipes());
            System.out.println("  Total size: " + stats.getSize() + " bytes");
            System.out.println("  Disk usage: " + stats.getUsage() + " bytes");
            System.out.println("  Duration: " + stats.getDuration() + "s");

        } catch (JScandirError e) {
            System.err.println("Error: " + e.getMessage() + " (code " + e.getCode() + ")");
            System.exit(1);
        }
    }
}