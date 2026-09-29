// C++ implementation that wraps the C API
#include "cxxscandir.h"
#include <cscandir.h>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <memory>

namespace cxxscandir {

// Helper to convert C string to std::string
static inline std::string to_string(const char* cstr) {
    return cstr ? std::string(cstr) : std::string();
}

// Helper to convert C string array to std::vector<std::string>
static std::vector<std::string> to_vector(const char* const* items, size_t len) {
    std::vector<std::string> result;
    if (!items || len == 0) return result;
    result.reserve(len);
    for (size_t i = 0; i < len; ++i) {
        result.emplace_back(items[i] ? items[i] : "");
    }
    return result;
}

// Helper to convert std::vector<std::string> to C string array
static char** to_c_array(const std::vector<std::string>& vec, size_t& out_len) {
    if (vec.empty()) {
        return nullptr;
    }
    char** arr = static_cast<char**>(std::malloc(vec.size() * sizeof(char*)));
    for (size_t i = 0; i < vec.size(); ++i) {
        arr[i] = strdup(vec[i].c_str());
    }
    return arr;
}

static void free_c_array(char** arr, size_t len) {
    if (!arr) return;
    for (size_t i = 0; i < len; ++i) {
        if (arr[i]) std::free(arr[i]);
    }
    std::free(arr);
}

// Convert C options to Go-like Options struct
void options_default(Options* opts) {
    cscandir_options_init(opts);
}

// Convert C Entry to Rust Entry
static Entry to_entry(const cscandir_entry* src) {
    Entry dst;
    dst.path = to_string(src->path);
    dst.is_symlink = src->is_symlink != 0;
    dst.is_dir = src->is_dir != 0;
    dst.is_file = src->is_file != 0;
    dst.ctime = src->ctime;
    dst.mtime = src->mtime;
    dst.atime = src->atime;
    dst.size = src->size;
    dst.has_ext = src->has_ext != 0;
    dst.mode = src->mode;
    dst.ino = src->ino;
    dst.dev = src->dev;
    dst.nlink = src->nlink;
    dst.blksize = src->blksize;
    dst.blocks = src->blocks;
    dst.uid = src->uid;
    dst.gid = src->gid;
    dst.rdev = src->rdev;
    return dst;
}

// Convert C Toc to Rust Toc
static Toc to_toc(const cscandir_toc* src) {
    Toc dst;
    if (!src) return dst;
    dst.dirs = to_vector(src->dirs.items, src->dirs.len);
    dst.files = to_vector(src->files.items, src->files.len);
    dst.symlinks = to_vector(src->symlinks.items, src->symlinks.len);
    dst.other = to_vector(src->other.items, src->other.len);
    dst.errors = to_vector(src->errors.items, src->errors.len);
    return dst;
}

// Convert C Statistics to Rust Statistics
static Statistics to_statistics(const cscandir_statistics* src) {
    Statistics dst;
    if (!src) return dst;
    dst.dirs = src->dirs;
    dst.files = src->files;
    dst.slinks = src->slinks;
    dst.hlinks = src->hlinks;
    dst.devices = src->devices;
    dst.pipes = src->pipes;
    dst.size = src->size;
    dst.usage = src->usage;
    dst.duration = src->duration;
    return dst;
}

// Main Collect function
Result<CollectResult> collect(const std::string& root, const Options& options) {
    // Prepare native options
    cscandir_options native;
    cscandir_options_init(&native);
    native.sorted = options.sorted;
    native.skip_hidden = options.skip_hidden;
    native.max_depth = options.max_depth;
    native.max_file_cnt = options.max_file_cnt;
    native.case_sensitive = options.case_sensitive;
    native.follow_links = options.follow_links;
    native.return_type = static_cast<uint32_t>(options.return_type);

    // Filter arrays
    char** dir_include = nullptr;
    char** dir_exclude = nullptr;
    char** file_include = nullptr;
    char** file_exclude = nullptr;
    size_t dir_include_len = 0;
    size_t dir_exclude_len = 0;
    size_t file_include_len = 0;
    size_t file_exclude_len = 0;

    // We need to allocate and free these arrays properly
    // For simplicity, we'll allocate them here and free after the call

    // Note: In a real implementation, you'd want to properly manage these
    // For now, we'll skip filter arrays for simplicity

    cscandir_entry_list entries;
    cscandir_entry_list_init(&entries);

    cscandir_string_list errors;
    cscandir_string_list_init(&errors);

    cscandir_statistics stats{};
    cscandir_error error;
    cscandir_error_init(&error);

    int32_t code = cscandir_collect(
        rust_str.c_str(),
        &native,
        &entries,
        &errs,
        &stats,
        &error
    );

    // Check error
    if (code != CSCANDIR_OK) {
        return Result<CollectResult>::Err(Error{
            static_cast<int32_t>(code),
            std::string(error.message ? error.message : "unknown error")
        });
    }

    // Convert results
    CollectResult result;
    result.statistics = to_statistics(&stats);
    result.errors = to_vector(errs.items, errs.len);
    result.entries.reserve(entries.len);
    for (size_t i = 0; i < entries.len; ++i) {
        result.entries.push_back(to_entry(&entries.entries[i]));
    }

    // Cleanup
    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errs);
    cscandir_free_error(&error);

    return Result<CollectResult>::Ok(std::move(result));
}

} // namespace cxxscandir