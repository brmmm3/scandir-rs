#define _POSIX_C_SOURCE 200809L
#include "cscandir.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

static double stat_ctime_seconds(const struct stat *info) {
#if defined(__APPLE__)
    return (double)info->st_ctimespec.tv_sec +
           (double)info->st_ctimespec.tv_nsec * 1e-9;
#elif defined(_WIN32)
    return (double)info->st_ctime;
#else
    return (double)info->st_ctim.tv_sec + (double)info->st_ctim.tv_nsec * 1e-9;
#endif
}

static int join_path(char *out, size_t size, const char *root, const char *relative) {
    const size_t root_len = strlen(root);
    const char *separator = root_len > 0 && root[root_len - 1] == '/' ? "" : "/";
    const int written = snprintf(out, size, "%s%s%s", root, separator, relative);
    return written >= 0 && (size_t)written < size;
}

int main(int argc, char **argv) {
    if (argc > 2) {
        fprintf(stderr, "Usage: %s [MOUNT_PATH]\n", argv[0]);
        return EXIT_FAILURE;
    }
    const char *root = argc == 2 ? argv[1] : "/mnt/smb";
    cscandir_options options;
    cscandir_options_init(&options);
    options.return_type = CSCANDIR_RETURN_EXT;

    cscandir_entry_list entries = {0};
    cscandir_string_list errors = {0};
    cscandir_error error = {0};
    int32_t status = cscandir_collect(root, &options, &entries, &errors, &error);
    if (status != CSCANDIR_OK) {
        fprintf(stderr, "scan failed (%" PRId32 "): %s\n", status,
                error.message ? error.message : "unknown error");
        cscandir_free_entry_list(&entries);
        cscandir_free_string_list(&errors);
        cscandir_free_error(&error);
        return EXIT_FAILURE;
    }

    printf("scanned %zu entries, %zu errors from %s\n", entries.len,
           errors.len, root);
    size_t ctime_ok = 0;
    size_t ctime_none = 0;
    size_t stat_checks = 0;
    double max_ctime_skew = 0.0;
    char full_path[4096];
    for (size_t i = 0; i < entries.len; ++i) {
        const cscandir_entry *entry = &entries.entries[i];
        if (!entry->is_file) {
            continue;
        }
        if (entry->ctime > 0.0) {
            ++ctime_ok;
        } else {
            ++ctime_none;
            printf("  CTIME-ZERO %-30s <<< REGRESSION?\n", entry->path);
        }

        double skew = 0.0;
        int has_stat_skew = 0;
        if (join_path(full_path, sizeof(full_path), root, entry->path)) {
            struct stat info;
            if (stat(full_path, &info) == 0) {
                skew = entry->ctime - stat_ctime_seconds(&info);
                if (skew < 0.0) {
                    skew = -skew;
                }
                if (skew > max_ctime_skew) {
                    max_ctime_skew = skew;
                }
                ++stat_checks;
                has_stat_skew = 1;
            }
        }
        printf("  %-30s ctime=%.6f", entry->path, entry->ctime);
        if (entry->has_ext) {
            printf(" mode=%" PRIu32 " ino=%" PRIu64, entry->mode, entry->ino);
        }
        if (has_stat_skew) {
            printf(" stat-skew=%.6fs", skew);
        }
        putchar('\n');
    }
    for (size_t i = 0; i < errors.len; ++i) {
        fprintf(stderr, "scan error: %s\n", errors.items[i]);
    }
    printf("summary: ctime %zu nonzero / %zu zero, stat checks=%zu, "
           "max ctime skew=%.6fs\n", ctime_ok, ctime_none, stat_checks,
           max_ctime_skew);
    if (stat_checks == 0) {
        fprintf(stderr, "warning: no entries could be checked against stat()\n");
    }

    const int failed = ctime_none != 0 || errors.len != 0;
    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
    return failed ? EXIT_FAILURE : EXIT_SUCCESS;
}