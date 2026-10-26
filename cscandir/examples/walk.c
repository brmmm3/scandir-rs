#define _POSIX_C_SOURCE 200809L
#include "cscandir.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static double monotonic_seconds(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
        perror("clock_gettime");
        exit(EXIT_FAILURE);
    }
    return (double)now.tv_sec + (double)now.tv_nsec * 1e-9;
}

int main(int argc, char **argv) {
    const char *root = "/usr";
    int have_root = 0;
    int extended = 0;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--ext") == 0) {
            extended = 1;
        } else if (!have_root) {
            root = argv[i];
            have_root = 1;
        } else {
            fprintf(stderr, "Usage: %s [ROOT] [--ext]\n", argv[0]);
            return EXIT_FAILURE;
        }
    }

    cscandir_options options;
    cscandir_options_init(&options);
    options.max_file_cnt = 100;
    if (extended) {
        options.return_type = CSCANDIR_RETURN_EXT;
    }

    cscandir_entry_list entries = {0};
    cscandir_string_list errors = {0};
    cscandir_error error = {0};
    const double start = monotonic_seconds();
    const int32_t status = cscandir_collect(root, &options, &entries, &errors, &error);
    const double elapsed = monotonic_seconds() - start;
    if (status != CSCANDIR_OK) {
        fprintf(stderr, "walk failed (%" PRId32 "): %s\n", status,
                error.message ? error.message : "unknown error");
    } else {
        printf("root=%s entries=%zu errors=%zu duration=%.6f seconds\n", root,
               entries.len, errors.len, elapsed);
        for (size_t i = 0; i < errors.len; ++i) {
            fprintf(stderr, "scan error: %s\n", errors.items[i]);
        }
        for (size_t i = 0; i < entries.len && i < 20; ++i) {
            const cscandir_entry *entry = &entries.entries[i];
            const char type = entry->is_symlink ? 'S' : entry->is_dir ? 'D' :
                              entry->is_file ? 'F' : '?';
            printf("%c %s size=%" PRIu64 "\n", type, entry->path, entry->size);
        }
        if (entries.len > 20) {
            printf("... (%zu entries total; showing first 20)\n", entries.len);
        }
    }

    const int failed = status != CSCANDIR_OK || errors.len != 0;
    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
    return failed ? EXIT_FAILURE : EXIT_SUCCESS;
}