#define _POSIX_C_SOURCE 200809L
#include "cscandir.h"

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static double now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t) != 0) {
        perror("clock_gettime");
        exit(EXIT_FAILURE);
    }
    return (double)t.tv_sec + (double)t.tv_nsec * 1e-9;
}

int main(int argc, char **argv) {
    if (argc < 2 || argc > 3) {
        fprintf(stderr, "Usage: %s ROOT [ITERATIONS]\n", argv[0]);
        return EXIT_FAILURE;
    }
    long iterations = 10;
    if (argc == 3) {
        char *end;
        errno = 0;
        iterations = strtol(argv[2], &end, 10);
        if (errno || end == argv[2] || *end || iterations < 1) {
            fputs("ITERATIONS must be a positive integer\n", stderr);
            return EXIT_FAILURE;
        }
    }
    const char *names[] = {"collect", "collect-no-output", "count", "collect-ext"};
    for (size_t mode = 0; mode < sizeof(names) / sizeof(names[0]); ++mode) {
        double elapsed = 0.0;
        size_t entries = 0;
        /* One warm-up per mode; timing includes conversion and deallocation. */
        for (long i = -1; i < iterations; ++i) {
            cscandir_options options;
            cscandir_options_init(&options);
            options.return_type = mode == 3 ? CSCANDIR_RETURN_EXT : CSCANDIR_RETURN_BASE;
            cscandir_entry_list result = {0};
            cscandir_statistics stats = {0};
            cscandir_error error = {0};
            const double start = now();
            int32_t code = mode == 2
                ? cscandir_count(argv[1], &options, &stats, NULL, &error)
                : cscandir_collect(argv[1], &options, mode == 1 ? NULL : &result,
                                   NULL, &error);
            if (code != CSCANDIR_OK) {
                fprintf(stderr, "%s: %s\n", names[mode], error.message ? error.message : "failed");
                cscandir_free_error(&error);
                return EXIT_FAILURE;
            }
            entries = mode == 2
                ? (size_t)stats.dirs + (size_t)stats.files + (size_t)stats.slinks
                : result.len;
            cscandir_free_entry_list(&result);
            cscandir_free_error(&error);
            if (i >= 0) {
                elapsed += now() - start;
            }
        }
        printf("%s: %.3f ms/scan (%ld iterations, %zu entries)\n",
               names[mode], elapsed * 1000.0 / (double)iterations, iterations, entries);
    }
    return EXIT_SUCCESS;
}