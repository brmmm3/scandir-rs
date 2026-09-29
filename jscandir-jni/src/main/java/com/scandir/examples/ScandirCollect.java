package com.scandir.examples;

import com.scandir.jni.JScandir;
import com.scandir.options.Options;
import com.scandir.options.ReturnType;
import com.scandir.result.CollectResult;
import com.scandir.result.Entry;
import com.scandir.error.JScandirException;

import java.nio.file.Path;
import java.nio.file.Paths;

/**
 * Example demonstrating the Collect function.
 * 
 * Usage: java -cp target/jscandir-jni-2.10.1.jar \
 *   -Djava.library.path=target/native \
 *   com.scandir.examples.ScandirCollect [directory]
 */
public class ScandirCollect {

    public static void main(String[] args) {
        String root = args.length > 0 ? args[0] : ".";

        Options options = Options.defaultOptions()
            .sorted(true)
            .skipHidden(true)
            .returnType(ReturnType.EXT);

        try {
            System.out.println("Scanning: " + root);
            CollectResult result = JScandir.collect(root, options);

            System.out.println("Found " + result.getEntries().size() + " entries");
            System.out.println("Directories: " + result.getStatistics().getDirs() +
                "  Files: " + result.getStatistics().getFiles() +
                "  Symlinks: " + result.getStatistics().getSlinks());
            System.out.println("Duration: " + result.getStatistics().getDuration() + "s\n");

            System.out.println("First 10 entries:");
            for (int i = 0; i < result.getEntries().size() && i < 10; i++) {
                Entry entry = result.getEntries().get(i);
                char kind = '?';
                if (entry.isDir()) kind = 'D';
                else if (entry.isFile()) kind = 'F';
                else if (entry.isSymlink()) kind = 'L';
                System.out.printf("  %c %s%n", kind, entry.getPath());
                if (entry.hasExt()) {
                    System.out.printf("    inode=%d mode=%o size=%d%n",
                        entry.getIno(), entry.getMode(), entry.getSize());
                }
            }

            if (!result.getErrors().isEmpty()) {
                System.out.println("\nScan errors:");
                for (String err : result.getErrors()) {
                    System.out.println("  " + err);
                }
            }

        } catch (JScandirException e) {
            System.err.println("Error: " + e.getMessage() + " (code " + e.getCode() + ")");
            System.exit(1);
        }
    }
}