// impl.cc - Implementation of the cxxscandir functions
#include "cxxscandir.h"
#include <cscandir.h>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <memory>

namespace cxxscandir {

static inline std::string to_string(const char* cstr) {
    return cstr ? std::string(cstr) : std::string();
}

static std::vector<std::string> to_vector(const char* const* items, size_t len) {
    std::vector<std::string> result;
    if (!items || len == 0) return result;
    result.reserve(len);
    for (size_t i = 0; i < len; ++i) {
        result.emplace_back(items[i] ? items[i] : "");
    }
    return result;
}

static char** to_c_array(const std::vector<std::string>& vec, size_t& out_len) {
    if (vec.empty()) {
        out_len = 0;
        return nullptr;
    }
    char** arr = static_cast<char**>(std::malloc(vec.size() * sizeof(char*)));
    for (size_t i = 0; i < vec.size(); ++i) {
        arr[i] = strdup(vec[i].c_str());
    }
    out_len = vec.size();
    return arr;
}

static void free_c_array(char** arr, size_t len) {
    if (!arr) return;
    for (size_t i = 0; i < len; ++i) {
        if (arr[i]) std::free(arr[i]);
    }
    std::free(arr);
}

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

Options options_default() {
    Options opts;
    cscandir_options_init(&opts);
    return opts;
}

Result<CollectResult> collect(const std::string& root, const Options& options) {
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

    // For simplicity, we'll handle filters in a basic way
    // A full implementation would need to properly allocate these

    cscandir_entry_list entries;
    cscandir_entry_list_init(&entries);

    cscandir_string_list errs;
    cscandir_string_list_init(&errs);

    cscandir_statistics stats{};
    cscandir_error error;
    cscandir_error_init(&error);

    int32_t code = cscandir_collect(
        root.c_str(),
        &native,
        &entries,
        &errs,
        &stats,
        &error
    );

    if (code != CSCANDIR_OK) {
        std::string msg = error.message ? error.message : "unknown error";
        cscandir_free_entry_list(&entries);
        cscandir_free_string_list(&errs);
        cscandir_free_error(&error);
        return Result<CollectResult>::Err(Error{static_cast<int32_t>(code), std::move(msg)});
    }

    CollectResult result;
    result.statistics = to_statistics(&stats);
    result.errors = to_vector(errs.items, errs.len);
    result.entries.reserve(entries.len);
    for (size_t i = 0; i < entries.len; ++i) {
        result.entries.push_back(to_entry(&entries.entries[i]));
    }

    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errs);
    cscandir_free_error(&error);

    return Result<CollectResult>::Ok(std::move(result));
}

Result<Statistics> count(const std::string& root, const Options& options) {
    cscandir_options native;
    cscandir_options_init(&native);
    native.sorted = options.sorted;
    native.skip_hidden = options.skip_hidden;
    native.max_depth = options.max_depth;
    native.max_file_cnt = options.max_file_cnt;
    native.case_sensitive = options.case_sensitive;
    native.follow_links = options.follow_links;
    native.return_type = static_cast<uint32_t>(options.return_type);

    cscandir_statistics stats{};
    cscandir_error error;
    cscandir_error_init(&error);

    int32_t code = cscandir_count(
        root.c_str(),
        &native,
        &stats,
        nullptr,
        &error
    );

    if (code != CSCANDIR_OK) {
        std::string msg = error.message ? error.message : "unknown error";
        cscandir_free_error(&error);
        return Result<Statistics>::Err(Error{static_cast<int32_t>(code), std::move(msg)});
    }

    Statistics result = to_statistics(&stats);
    cscandir_free_error(&error);
    return Result<Statistics>::Ok(std::move(result));
}

Result<WalkResult> walk(const std::string& root, const Options& options) {
    cscandir_options native;
    cscandir_options_init(&native);
    native.sorted = options.sorted;
    native.skip_hidden = options.skip_hidden;
    native.max_depth = options.max_depth;
    native.max_file_cnt = options.max_file_cnt;
    native.case_sensitive = options.case_sensitive;
    native.follow_links = options.follow_links;
    native.return_type = static_cast<uint32_t>(options.return_type);

    cscandir_toc toc;
    cscandir_toc_init(&toc);

    cscandir_walk_entry_list roots;
    cscandir_walk_entry_list_init(&roots);

    cscandir_statistics stats{};
    cscandir_error error;
    cscandir_error_init(&error);

    int32_t code = cscandir_walk(
        root.c_str(),
        &native,
        &toc,
        &roots,
        &stats,
        &error
    );

    if (code != CSCANDIR_OK) {
        std::string msg = error.message ? error.message : "unknown error";
        cscandir_free_toc(&toc);
        cscandir_free_walk_entry_list(&roots);
        cscandir_free_error(&error);
        return Result<WalkResult>::Err(Error{static_cast<int32_t>(code), std::move(msg)});
    }

    WalkResult result;
    result.toc = to_toc(&toc);
    result.statistics = to_statistics(&stats);
    result.roots.reserve(roots.len);
    for (size_t i = 0; i < roots.len; ++i) {
        WalkEntry entry;
        entry.path = to_string(roots.entries[i].path);
        entry.toc = to_toc(&roots.entries[i].toc);
        result.roots.push_back(std::move(entry));
    }

    cscandir_free_toc(&toc);
    cscandir_free_walk_entry_list(&roots);
    cscandir_free_error(&error);

    return Result<WalkResult>::Ok(std::move(result));
}

} // namespace cxxscandir