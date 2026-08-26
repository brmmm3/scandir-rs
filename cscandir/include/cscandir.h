#ifndef CSCANDIR_H
#define CSCANDIR_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum {
    CSCANDIR_OK = 0,
    CSCANDIR_ERR_INVALID_ARGUMENT = 1,
    CSCANDIR_ERR_INVALID_UTF8 = 2,
    CSCANDIR_ERR_SCAN = 3,
    CSCANDIR_ERR_NUL_BYTE = 4
};

typedef enum cscandir_return_type {
    CSCANDIR_RETURN_BASE = 0,
    CSCANDIR_RETURN_EXT = 1
} cscandir_return_type;

typedef struct cscandir_options {
    uint8_t sorted;
    uint8_t skip_hidden;
    size_t max_depth;
    size_t max_file_cnt;

    const char* const* dir_include;
    size_t dir_include_len;

    const char* const* dir_exclude;
    size_t dir_exclude_len;

    const char* const* file_include;
    size_t file_include_len;

    const char* const* file_exclude;
    size_t file_exclude_len;

    uint8_t case_sensitive;
    uint8_t follow_links;
    uint32_t return_type;
} cscandir_options;

typedef struct cscandir_entry {
    char* path;
    uint8_t is_symlink;
    uint8_t is_dir;
    uint8_t is_file;

    double ctime;
    double mtime;
    double atime;
    uint64_t size;

    uint8_t has_ext;
    uint32_t mode;
    uint64_t ino;
    uint64_t dev;
    uint64_t nlink;
    uint64_t blksize;
    uint64_t blocks;
    uint32_t uid;
    uint32_t gid;
    uint64_t rdev;
} cscandir_entry;

typedef struct cscandir_entry_list {
    cscandir_entry* entries;
    size_t len;
    size_t capacity;
} cscandir_entry_list;

typedef struct cscandir_string_list {
    char** items;
    size_t len;
    size_t capacity;
} cscandir_string_list;

typedef struct cscandir_error {
    int32_t code;
    char* message;
} cscandir_error;

typedef struct cscandir_statistics {
    int32_t dirs;
    int32_t files;
    int32_t slinks;
    int32_t hlinks;
    int32_t devices;
    int32_t pipes;
    uint64_t size;
    uint64_t usage;
    double duration;
} cscandir_statistics;

/* Initialize options with defaults used by scandir::Scandir. */
void cscandir_options_init(cscandir_options* options);

/*
 * Collect directory entries.
 * - root_path must be a valid UTF-8 C string.
 * - out_entries and out_errors are optional; pass NULL if not needed.
 * - free outputs with cscandir_free_entry_list / cscandir_free_string_list.
 */
int32_t cscandir_collect(
    const char* root_path,
    const cscandir_options* options,
    cscandir_entry_list* out_entries,
    cscandir_string_list* out_errors,
    cscandir_error* out_error
);

/*
 * Collect aggregate directory statistics.
 * - out_stats is optional; pass NULL if not needed.
 * - free out_errors with cscandir_free_string_list.
 */
int32_t cscandir_count(
    const char* root_path,
    const cscandir_options* options,
    cscandir_statistics* out_stats,
    cscandir_string_list* out_errors,
    cscandir_error* out_error
);

void cscandir_free_entry_list(cscandir_entry_list* list);
void cscandir_free_string_list(cscandir_string_list* list);
void cscandir_free_error(cscandir_error* error);

#ifdef __cplusplus
}
#endif

#endif
