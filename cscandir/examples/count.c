#include "cscandir.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    const char *root = "/tmp";
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
    const char *excluded_dirs[] = {"dir0", "dir1"};
    options.dir_exclude = excluded_dirs;
    options.dir_exclude_len = sizeof(excluded_dirs) / sizeof(excluded_dirs[0]);
    if (extended) {
        options.return_type = CSCANDIR_RETURN_EXT;
    }

    cscandir_statistics stats = {0};
    cscandir_string_list errors = {0};
    cscandir_error error = {0};
    const int32_t status = cscandir_count(root, &options, &stats, &errors, &error);
    if (status != CSCANDIR_OK) {
        fprintf(stderr, "count failed (%" PRId32 "): %s\n", status,
                error.message ? error.message : "unknown error");
    } else {
        printf("root=%s metadata=%s\n", root,
               options.return_type == CSCANDIR_RETURN_EXT ? "extended" : "base");
        printf("dirs=%" PRId32 " files=%" PRId32 " symlinks=%" PRId32 "\n",
               stats.dirs, stats.files, stats.slinks);
        if (options.return_type == CSCANDIR_RETURN_EXT) {
            printf("hardlinks=%" PRId32 " devices=%" PRId32 " pipes=%" PRId32 "\n",
                   stats.hlinks, stats.devices, stats.pipes);
            printf("size=%" PRIu64 " usage=%" PRIu64 " bytes\n", stats.size,
                   stats.usage);
        }
        printf("duration=%.6f seconds\n", stats.duration);
    }
    for (size_t i = 0; i < errors.len; ++i) {
        fprintf(stderr, "scan error: %s\n", errors.items[i]);
    }

    const int failed = status != CSCANDIR_OK || errors.len != 0;
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
    return failed ? EXIT_FAILURE : EXIT_SUCCESS;
}