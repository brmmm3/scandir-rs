#define _POSIX_C_SOURCE 200809L
#include "cscandir.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static void path_join(char *out, size_t size, const char *root, const char *name) {
    int n = snprintf(out, size, "%s/%s", root, name);
    assert(n >= 0 && (size_t)n < size);
}

static void create_file(const char *root, const char *name) {
    char path[1024];
    path_join(path, sizeof(path), root, name);
    FILE *file = fopen(path, "wb");
    assert(file);
    assert(fwrite("abc", 1, 3, file) == 3);
    assert(fclose(file) == 0);
}

static int contains(const cscandir_entry_list *entries, const char *path) {
    for (size_t i = 0; i < entries->len; ++i) {
        if (strcmp(entries->entries[i].path, path) == 0) {
            return 1;
        }
    }
    return 0;
}

static void expect_failure(const char *root, const cscandir_options *options, int32_t code) {
    /* Poisoning writable outputs detects implementations that read/free them
     * before initialization. No input pointer is invalid or dangling. */
    cscandir_entry_list entries;
    cscandir_string_list errors;
    cscandir_error error;
    cscandir_statistics stats;
    memset(&entries, 0xa5, sizeof(entries));
    memset(&errors, 0xa5, sizeof(errors));
    memset(&error, 0xa5, sizeof(error));
    assert(cscandir_collect(root, options, &entries, &errors, &error) == code);
    assert(error.code == code && error.message && error.message[0]);
    assert(!entries.entries && entries.len == 0 && entries.capacity == 0);
    assert(!errors.items && errors.len == 0 && errors.capacity == 0);
    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
    cscandir_free_error(&error);

    memset(&stats, 0xa5, sizeof(stats));
    memset(&errors, 0xa5, sizeof(errors));
    memset(&error, 0xa5, sizeof(error));
    assert(cscandir_count(root, options, &stats, &errors, &error) == code);
    assert(error.code == code && error.message);
    assert(stats.dirs == 0 && stats.files == 0 && stats.slinks == 0);
    assert(stats.size == 0 && stats.usage == 0 && stats.duration == 0.0);
    assert(!errors.items && errors.len == 0);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
}

static cscandir_entry_list collect(const char *root, const cscandir_options *options) {
    cscandir_entry_list entries;
    cscandir_string_list errors;
    cscandir_error error;
    memset(&error, 0xa5, sizeof(error));
    assert(cscandir_collect(root, options, &entries, &errors, &error) == CSCANDIR_OK);
    assert(error.code == CSCANDIR_OK && !error.message);
    assert(errors.len == 0);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
    return entries;
}

static void test_invalid_inputs(const char *root) {
    expect_failure(NULL, NULL, CSCANDIR_ERR_INVALID_ARGUMENT);
    const char bad_utf8[] = {(char)0xff, 0};
    expect_failure(bad_utf8, NULL, CSCANDIR_ERR_INVALID_UTF8);
    expect_failure("/definitely/not/a/cscandir/test/path", NULL, CSCANDIR_ERR_SCAN);
    cscandir_options options;
    cscandir_options_init(&options);
    options.return_type = 99;
    expect_failure(root, &options, CSCANDIR_ERR_INVALID_ARGUMENT);
    cscandir_options_init(&options);
    options.file_include_len = 1;
    expect_failure(root, &options, CSCANDIR_ERR_INVALID_ARGUMENT);
    const char *null_item[] = {NULL};
    options.file_include = null_item;
    expect_failure(root, &options, CSCANDIR_ERR_INVALID_ARGUMENT);
    const char *bad_item[] = {bad_utf8};
    options.file_include = bad_item;
    expect_failure(root, &options, CSCANDIR_ERR_INVALID_UTF8);
    const char *bad_glob[] = {"["};
    options.file_include = bad_glob;
    expect_failure(root, &options, CSCANDIR_ERR_INVALID_ARGUMENT);
    options.file_include_len = SIZE_MAX;
    expect_failure(root, &options, CSCANDIR_ERR_INVALID_ARGUMENT);
    assert(cscandir_collect(NULL, NULL, NULL, NULL, NULL) == CSCANDIR_ERR_INVALID_ARGUMENT);
    assert(cscandir_count(NULL, NULL, NULL, NULL, NULL) == CSCANDIR_ERR_INVALID_ARGUMENT);
}

static void test_collect_and_count(const char *root) {
    cscandir_entry_list entries = collect(root, NULL);
    assert(entries.len == 8);
    assert(contains(&entries, ".hidden") && contains(&entries, "included/kept.txt"));
    for (size_t i = 0; i < entries.len; ++i) {
        assert(entries.entries[i].has_ext == 0);
        assert(entries.entries[i].mode == 0 && entries.entries[i].ino == 0);
    }
    cscandir_free_entry_list(&entries);
    cscandir_free_entry_list(&entries);
    assert(!entries.entries && entries.len == 0 && entries.capacity == 0);

    cscandir_options options;
    cscandir_options_init(&options);
    assert(options.max_depth == SIZE_MAX && options.max_file_cnt == SIZE_MAX);
    options.sorted = 1;
    options.return_type = CSCANDIR_RETURN_EXT;
    entries = collect(root, &options);
    assert(entries.len == 8);
    const char *root_name = strrchr(root, '/');
    assert(strcmp(entries.entries[0].path, root_name ? root_name + 1 : root) == 0);
    const char *sorted[] = {".hidden", "a.txt", "b.TXT", "excluded",
                            "excluded/omitted.txt", "included", "included/kept.txt"};
    for (size_t i = 1; i < entries.len; ++i) {
        const cscandir_entry *e = &entries.entries[i];
        assert(strcmp(e->path, sorted[i - 1]) == 0);
        assert(e->has_ext);
        assert(e->mode != 0 && e->nlink > 0);
        assert(e->mtime > 0.0 && e->ctime > 0.0 && e->atime > 0.0);
        if (e->is_file) {
            assert(e->size == 3);
        }
    }
    cscandir_free_entry_list(&entries);
    cscandir_statistics stats;
    cscandir_string_list errors;
    cscandir_error error;
    assert(cscandir_count(root, &options, &stats, &errors, &error) == CSCANDIR_OK);
    assert(stats.dirs == 2 && stats.files == 5 && stats.slinks == 0);
    assert(stats.size >= 15 && stats.duration >= 0.0 && errors.len == 0);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
    assert(cscandir_collect(root, NULL, NULL, NULL, NULL) == CSCANDIR_OK);
    assert(cscandir_count(root, NULL, NULL, NULL, NULL) == CSCANDIR_OK);
    /* Repeated use requires explicit frees, but no manual reinitialization. */
    for (int i = 0; i < 30; ++i) {
        assert(cscandir_collect(root, NULL, &entries, &errors, &error) == CSCANDIR_OK);
        cscandir_free_entry_list(&entries);
        cscandir_free_string_list(&errors);
        cscandir_free_error(&error);
    }
}

static void test_filters(const char *root) {
    cscandir_options options;
    cscandir_options_init(&options);
    options.skip_hidden = 1;
    const char *include[] = {"included/**"};
    const char *exclude[] = {"excluded/**"};
    const char *files[] = {"*.txt"};
    options.dir_include = include;
    options.dir_include_len = 1;
    options.dir_exclude = exclude;
    options.dir_exclude_len = 1;
    options.file_include = files;
    options.file_include_len = 1;
    options.case_sensitive = 1;
    cscandir_entry_list entries = collect(root, &options);
    assert(entries.len == 4 && contains(&entries, "a.txt"));
    assert(contains(&entries, "included") && contains(&entries, "included/kept.txt"));
    cscandir_free_entry_list(&entries);
    const char *file_exclude[] = {"kept*"};
    options.file_exclude = file_exclude;
    options.file_exclude_len = 1;
    entries = collect(root, &options);
    assert(entries.len == 3 && !contains(&entries, "included/kept.txt"));
    cscandir_free_entry_list(&entries);
    cscandir_options_init(&options);
    options.max_depth = 1;
    entries = collect(root, &options);
    assert(entries.len == 6);
    cscandir_free_entry_list(&entries);
    options.max_depth = 0;
    options.max_file_cnt = 1;
    entries = collect(root, &options);
    assert(entries.len > 0 && entries.len < 8);
    cscandir_free_entry_list(&entries);
}

int main(void) {
    cscandir_options_init(NULL);
    cscandir_free_entry_list(NULL);
    cscandir_free_string_list(NULL);
    cscandir_free_error(NULL);
    char root[] = "/tmp/cscandir-api-XXXXXX";
    assert(mkdtemp(root));
    char path[1024];
    path_join(path, sizeof(path), root, "included");
    assert(mkdir(path, 0700) == 0);
    path_join(path, sizeof(path), root, "excluded");
    assert(mkdir(path, 0700) == 0);
    const char *files[] = {"a.txt", "b.TXT", ".hidden", "included/kept.txt", "excluded/omitted.txt"};
    for (size_t i = 0; i < sizeof(files) / sizeof(files[0]); ++i) {
        create_file(root, files[i]);
    }
    test_invalid_inputs(root);
    test_collect_and_count(root);
    test_filters(root);
    for (size_t i = 0; i < sizeof(files) / sizeof(files[0]); ++i) {
        path_join(path, sizeof(path), root, files[i]);
        assert(unlink(path) == 0);
    }
    path_join(path, sizeof(path), root, "included");
    assert(rmdir(path) == 0);
    path_join(path, sizeof(path), root, "excluded");
    assert(rmdir(path) == 0);
    assert(rmdir(root) == 0);
    puts("C API regression tests passed");
    return EXIT_SUCCESS;
}