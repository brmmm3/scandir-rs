package com.scandir.examples;

import com.scandir.jni.JScandir;
import com.scandir.options.Options;
import com.scandir.options.ReturnType;
import com.scandir.result.CollectResult;
import com.scandir.result.Entry;

import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.FileTime;

/** Checks collected ctime values on a CIFS mount and cross-checks unix:ctime. */
public final class CifsCtimeCheck {
    private CifsCtimeCheck() { }

    public static void main(String[] args) throws Exception {
        if (args.length > 1) {
            System.err.println("Usage: CifsCtimeCheck [MOUNT_PATH]");
            System.exit(2);
        }
        Path root = Path.of(args.length == 1 ? args[0] : "/mnt/smb");
        if (!Files.isDirectory(root)) {
            System.err.println("Not a readable directory: " + root);
            System.exit(2);
        }
        try {
            String filesystem = Files.getFileStore(root).type();
            if (!filesystem.toLowerCase(java.util.Locale.ROOT).contains("cifs") &&
                    !filesystem.toLowerCase(java.util.Locale.ROOT).contains("smb")) {
                System.err.println("Warning: " + root + " is on " + filesystem + ", not a detected CIFS/SMB mount");
            }
        } catch (Exception e) {
            System.err.println("Warning: could not identify filesystem for " + root + ": " + e.getMessage());
        }
        CollectResult result = JScandir.collect(root,
                Options.defaultOptions().sorted(true).returnType(ReturnType.EXT));
        long ctimeOk = 0, ctimeZero = 0, statChecks = 0;
        double maxSkew = 0;
        System.out.printf("scanned %d entries, %d errors from %s%n", result.getEntries().size(),
                result.getErrors().size(), root);
        for (Entry entry : result.getEntries()) {
            if (!entry.isFile()) continue;
            if (entry.getCtime() > 0.0) ctimeOk++;
            else {
                ctimeZero++;
                System.out.printf("  CTIME-ZERO %-30s <<< REGRESSION?%n", entry.getPath());
            }
            Path full = root.resolve(entry.getPath());
            try {
                FileTime statCtime = (FileTime) Files.getAttribute(full, "unix:ctime");
                double skew = Math.abs(entry.getCtime() - statCtime.toInstant().toEpochMilli() / 1000.0);
                maxSkew = Math.max(maxSkew, skew);
                statChecks++;
                System.out.printf("  %-30s ctime=%.6f stat-skew=%.6fs%n", entry.getPath(), entry.getCtime(), skew);
            } catch (Exception e) {
                System.out.printf("  %-30s ctime=%.6f stat-check-unavailable: %s%n",
                        entry.getPath(), entry.getCtime(), e.getMessage());
            }
        }
        result.getErrors().forEach(error -> System.err.println("scan error: " + error));
        System.out.printf("summary: ctime %d nonzero / %d zero, stat checks=%d, max ctime skew=%.6fs%n",
                ctimeOk, ctimeZero, statChecks, maxSkew);
        if (statChecks == 0) System.err.println("warning: no entries could be checked against unix:ctime");
        if (statChecks == 0 || ctimeZero > 0 || !result.getErrors().isEmpty()) System.exit(1);
    }
}