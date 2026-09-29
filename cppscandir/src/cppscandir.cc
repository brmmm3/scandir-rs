#include "cppscandir.h"
#include <cscandir.h>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <memory>

namespace cppscandir {

namespace detail {

// Helper to convert C string to std::string
inline std::string to_string(const char* cstr) {
    return cstr ? std::string(cstr) : std::string();
}

// Helper to convert C string array to std::vector<std::string>
inline std::vector<std::string> to_vector(const char* const* items, size_t len) {
    std::vector<std::string> result;
    if (!items || len == 0) return result;
    result.reserve(len);
    for (size_t i = 0; i < len; ++i) {
        result.emplace_back(items[i] ? items[i] : "");
    }
    return result;
}

// Helper to convert std::vector<std::string> to C string array
inline char** to_c_array(const std::vector<std::string>& vec, size_t& out_len) {
    if (vec.empty()) {
        out_len = 0;
        return nullptr;
    }
    char** arr = static_cast<char**>(std::malloc(vec.size() * sizeof(char*)));
    for (size_t i = 0; i < vec.size(); ++i) {
        arr[i] = ::strdup(vec[i].c_str());
    }
    out_len = vec.size();
    return arr;
}

inline void free_c_array(char** arr, size_t len) {
    if (!arr) return;
    for (size_t i = 0; i < len; ++i) {
        if (arr[i]) std::free(arr[i]);
    }
    std::free(arr);
}

// RAII wrapper for C string array
class CStringArray {
public:
    CStringArray() : ptrs_(nullptr), len_(0) {}
    
    explicit CStringArray(const std::vector<std::string>& vec) {
        ptrs_ = to_c_array(vec, len_);
    }
    
    ~CStringArray() {
        free_c_array(ptrs_, len_);
    }
    
    CStringArray(CStringArray&& other) noexcept : ptrs_(other.ptrs_), len_(other.len_) {
        other.ptrs_ = nullptr;
        other.len_ = 0;
    }
    
    CStringArray& operator=(CStringArray&& other) noexcept {
        if (this != &other) {
            free_c_array(ptrs_, len_);
            ptrs_ = other.ptrs_;
            len_ = other.len_;
            other.ptrs_ = nullptr;
            other.len_ = 0;
        }
        return *this;
    }
    
    CStringArray(const CStringArray&) = delete;
    CStringArray& operator=(const CStringArray&) = delete;
    
    const char* const* data() const noexcept { return ptrs_; }
    size_t size() const noexcept { return len_; }
    
private:
    char** ptrs_;
    size_t len_;
};

inline std::vector<std::string> to_vector(const cscandir_string_list& list) {
    if (!list.items || list.len == 0) return {};
    std::vector<std::string> result;
    result.reserve(list.len);
    for (size_t i = 0; i < list.len; ++i) {
        result.emplace_back(list.items[i] ? list.items[i] : "");
    }
    return result;
}

inline Entry to_entry(const cscandir_entry& src) {
    Entry dst;
    dst.path = to_string(src.path);
    dst.is_symlink = src.is_symlink != 0;
    dst.is_dir = src.is_dir != 0;
    dst.is_file = src.is_file != 0;
    dst.ctime = src.ctime;
    dst.mtime = src.mtime;
    dst.atime = src.atime;
    dst.size = src.size;
    dst.has_ext = src.has_ext != 0;
    dst.mode = src.mode;
    dst.ino = src.ino;
    dst.dev = src.dev;
    dst.nlink = src.nlink;
    dst.blksize = src.blksize;
    dst.blocks = src.blocks;
    dst.uid = src.uid;
    dst.gid = src.gid;
    dst.rdev = src.rdev;
    return dst;
}

inline std::vector<std::string> to_vector(const cscandir_string_list* list) {
    if (!list || !list->items || list->len == 0) return {};
    return to_vector(*list);
}

inline Toc to_toc(const cscandir_toc& src) {
    Toc dst;
    dst.dirs = to_vector(&src.dirs);
    dst.files = to_vector(&src.files);
    dst.symlinks = to_vector(&src.symlinks);
    dst.other = to_vector(&src.other);
    dst.errors = to_vector(&src.errors);
    return dst;
}

inline Statistics to_statistics(const cscandir_statistics& src) {
    Statistics dst;
    dst.dirs = src.dirs;
    dst.files = src.files;
    dst.slinks = src.slinks;
    dst.hlinks = src.hlinks;
    dst.devices = src.devices;
    dst.pipes = src.pipes;
    dst.size = src.size;
    dst.usage = src.usage;
    dst.duration = src.duration;
    return dst;
}

inline void check_error(int32_t code, const cscandir_error& error) {
    if (code != CSCANDIR_OK) {
        std::string msg = error.message ? error.message : "unknown error";
        throw CppScandirError(code, std::move(msg));
    }
}

} // namespace detail

CollectResult collect(const std::string& root, const Options& options) {
    cscandir_options native{};
    cscandir_options_init(&native);
    
    native.sorted = options.sorted;
    native.skip_hidden = options.skip_hidden;
    native.max_depth = options.max_depth;
    native.max_file_cnt = options.max_file_cnt;
    native.case_sensitive = options.case_sensitive;
    native.follow_links = options.follow_links;
    native.return_type = static_cast<uint32_t>(options.return_type);

    detail::CStringArray dir_include(options.dir_include);
    detail::CStringArray dir_exclude(options.dir_exclude);
    detail::CStringArray file_include(options.file_include);
    detail::CStringArray file_exclude(options.file_exclude);

    native.dir_include = dir_include.data();
    native.dir_include_len = dir_include.size();
    native.dir_exclude = dir_exclude.data();
    native.dir_exclude_len = dir_exclude.size();
    native.file_include = file_include.data();
    native.file_include_len = file_include.size();
    native.file_exclude = file_exclude.data();
    native.file_exclude_len = file_exclude.size();

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

    detail::check_error(code, error);

    CollectResult result;
    result.statistics = detail::to_statistics(stats);
    result.errors = detail::to_vector(&errs);
    
    if (entries.len > 0) {
        result.entries.reserve(entries.len);
        for (size_t i = 0; i < entries.len; ++i) {
            result.entries.push_back(detail::to_entry(entries.entries[i]));
        }
    }

    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errs);
    cscandir_free_error(&error);

    return result;
}

Statistics count(const std::string& root, const Options& options) {
    cscandir_options native{};
    cscandir_options_init(&native);
    
    native.sorted = options.sorted;
    native.skip_hidden = options.skip_hidden;
    native.max_depth = options.max_depth;
    native.max_file_cnt = options.max_file_cnt;
    native.case_sensitive = options.case_sensitive;
    native.follow_links = options.follow_links;
    native.return_type = static_cast<uint32_t>(options.return_type);

    detail::CStringArray dir_include(options.dir_include);
    detail::CStringArray dir_exclude(options.dir_exclude);
    detail::CStringArray file_include(options.file_include);
    detail::CStringArray file_exclude(options.file_exclude);

    native.dir_include = dir_include.data();
    native.dir_include_len = dir_include.size();
    native.dir_exclude = dir_exclude.data();
    native.dir_exclude_len = dir_exclude.size();
    native.file_include = file_include.data();
    native.file_include_len = file_include.size();
    native.file_exclude = file_exclude.data();
    native.file_exclude_len = file_exclude.size();

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

    detail::check_error(code, error);

    Statistics result = detail::to_statistics(stats);
    cscandir_free_error(&error);
    return result;
}

WalkResult walk(const std::string& root, const Options& options) {
    cscandir_options native{};
    cscandir_options_init(&native);
    
    native.sorted = options.sorted;
    native.skip_hidden = options.skip_hidden;
    native.max_depth = options.max_depth;
    native.max_file_cnt = options.max_file_cnt;
    native.case_sensitive = options.case_sensitive;
    native.follow_links = options.follow_links;
    native.return_type = static_cast<uint32_t>(options.return_type);

    detail::CStringArray dir_include(options.dir_include);
    detail::CStringArray dir_exclude(options.dir_exclude);
    detail::CStringArray file_include(options.file_include);
    detail::CStringArray file_exclude(options.file_exclude);

    native.dir_include = dir_include.data();
    native.dir_include_len = dir_include.size();
    native.dir_exclude = dir_exclude.data();
    native.dir_exclude_len = dir_exclude.size();
    native.file_include = file_include.data();
    native.file_include_len = file_include.size();
    native.file_exclude = file_exclude.data();
    native.file_exclude_len = file_exclude.size();

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

    detail::check_error(code, error);

    WalkResult result;
    result.toc = detail::to_toc(toc);
    result.statistics = detail::to_statistics(stats);
    
    if (roots.len > 0) {
        result.roots.reserve(roots.len);
        for (size_t i = 0; i < roots.len; ++i) {
            WalkEntry entry;
            entry.dir = detail::to_string(roots.entries[i].path);
            entry.toc = detail::to_toc(roots.entries[i].toc);
            result.roots.push_back(std::move(entry));
        }
    }

    cscandir_free_toc(&toc);
    cscandir_free_walk_entry_list(&roots);
    cscandir_free_error(&error);

    return result;
}

} // namespace cppscandir