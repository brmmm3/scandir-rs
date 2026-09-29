// cscandir.hpp - header-only C++17 wrapper around the cscandir C ABI.
//
// The C API hands ownership back to the caller through out-structs that must
// be released by hand. This wrapper hides that entirely behind RAII: every
// call returns standard C++ types that clean themselves up.
//
//   #include "cscandir.hpp"
//
//   cscandir::Options opts;
//   opts.skip_hidden(true).return_type(cscandir::ReturnType::ext);
//   auto result = cscandir::collect("/usr", opts);
//
// Errors are reported by throwing cscandir::Error. Errors encountered per
// entry during the scan (unreadable subdirectories, and so on) are not
// exceptional and are returned in `result.errors`.

#ifndef CSCANDIR_HPP
#define CSCANDIR_HPP

#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

#include "cscandir.h"

namespace cscandir {

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Error code reported by the C ABI, mirrored so callers need not include
/// cscandir.h to switch on them.
enum class ErrorCode : std::int32_t {
    ok = CSCANDIR_OK,
    invalid_argument = CSCANDIR_ERR_INVALID_ARGUMENT,
    invalid_utf8 = CSCANDIR_ERR_INVALID_UTF8,
    scan = CSCANDIR_ERR_SCAN,
    nul_byte = CSCANDIR_ERR_NUL_BYTE,
};

/// Thrown when a cscandir call fails. `code()` carries the C error code.
class Error : public std::runtime_error {
  public:
    Error(ErrorCode code, const std::string& what)
        : std::runtime_error(what), code_(code) {}

    ErrorCode code() const noexcept { return code_; }

  private:
    ErrorCode code_;
};

// ---------------------------------------------------------------------------
// Plain data types
// ---------------------------------------------------------------------------

enum class ReturnType : std::uint32_t {
    base = CSCANDIR_RETURN_BASE,
    ext = CSCANDIR_RETURN_EXT,
};

struct Statistics {
    std::int32_t dirs = 0;
    std::int32_t files = 0;
    std::int32_t slinks = 0;
    std::int32_t hlinks = 0;
    std::int32_t devices = 0;
    std::int32_t pipes = 0;
    std::uint64_t size = 0;
    std::uint64_t usage = 0;
    double duration = 0.0;
};

/// A single directory entry. The extended fields are only populated when
/// Options::return_type(ReturnType::ext) was requested; check has_ext().
struct Entry {
    std::string path;
    bool is_symlink = false;
    bool is_dir = false;
    bool is_file = false;

    /// Times in seconds since the Unix epoch.
    double ctime = 0.0;
    double mtime = 0.0;
    double atime = 0.0;

    std::uint64_t size = 0;

    bool has_ext = false;
    std::uint32_t mode = 0;
    std::uint64_t ino = 0;
    std::uint64_t dev = 0;
    std::uint64_t nlink = 0;
    std::uint64_t blksize = 0;
    std::uint64_t blocks = 0;
    std::uint32_t uid = 0;
    std::uint32_t gid = 0;
    std::uint64_t rdev = 0;
};

/// Entry names grouped by type, relative to the directory they came from.
struct Toc {
    std::vector<std::string> dirs;
    std::vector<std::string> files;
    std::vector<std::string> symlinks;
    std::vector<std::string> other;
    std::vector<std::string> errors;

    [[nodiscard]] bool empty() const noexcept {
        return dirs.empty() && files.empty() && symlinks.empty() &&
               other.empty() && errors.empty();
    }
};

/// One directory visited by walk(), with the entries it contained.
struct WalkEntry {
    /// Directory name relative to the scan root; empty for the root itself.
    /// Named `dir` rather than `path` so it does not shadow a
    /// std::filesystem::path in the surrounding scope.
    std::string dir;
    Toc toc;
};

struct CollectResult {
    std::vector<Entry> entries;
    /// Non-fatal per-path errors encountered during the scan.
    std::vector<std::string> errors;
    Statistics stats;
};

struct WalkResult {
    /// All entries merged, with fully qualified paths.
    Toc toc;
    /// Per-directory breakdown.
    std::vector<WalkEntry> roots;
    Statistics stats;
};

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Scan options. Defaults match scandir::Scandir; the filter and max_* members
/// accept 0 to mean "unlimited", matching the Rust API.
class Options {
  public:
    Options() { cscandir_options_init(&native_); }

    Options& sorted(bool value) noexcept {
        native_.sorted = value ? 1U : 0U;
        return *this;
    }
    Options& skip_hidden(bool value) noexcept {
        native_.skip_hidden = value ? 1U : 0U;
        return *this;
    }
    Options& max_depth(std::size_t value) noexcept {
        native_.max_depth = value;
        return *this;
    }
    Options& max_file_cnt(std::size_t value) noexcept {
        native_.max_file_cnt = value;
        return *this;
    }
    Options& dir_include(std::vector<std::string> patterns) {
        dir_include_ = std::move(patterns);
        return *this;
    }
    Options& dir_exclude(std::vector<std::string> patterns) {
        dir_exclude_ = std::move(patterns);
        return *this;
    }
    Options& file_include(std::vector<std::string> patterns) {
        file_include_ = std::move(patterns);
        return *this;
    }
    Options& file_exclude(std::vector<std::string> patterns) {
        file_exclude_ = std::move(patterns);
        return *this;
    }
    Options& case_sensitive(bool value) noexcept {
        native_.case_sensitive = value ? 1U : 0U;
        return *this;
    }
    Options& follow_links(bool value) noexcept {
        native_.follow_links = value ? 1U : 0U;
        return *this;
    }
    Options& return_type(ReturnType value) noexcept {
        native_.return_type = static_cast<std::uint32_t>(value);
        return *this;
    }

    /// Access the underlying C struct, for interoperating with the C API.
    const cscandir_options& native() const noexcept { return native_; }

    // Pattern getters, used by the entry points to build the C pointer arrays.
    const std::vector<std::string>& dir_include() const noexcept {
        return dir_include_;
    }
    const std::vector<std::string>& dir_exclude() const noexcept {
        return dir_exclude_;
    }
    const std::vector<std::string>& file_include() const noexcept {
        return file_include_;
    }
    const std::vector<std::string>& file_exclude() const noexcept {
        return file_exclude_;
    }

  private:
    cscandir_options native_{};
    std::vector<std::string> dir_include_{};
    std::vector<std::string> dir_exclude_{};
    std::vector<std::string> file_include_{};
    std::vector<std::string> file_exclude_{};
};

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/// Scan `root` and return an entry per result (scandir::Scandir).
[[nodiscard]] CollectResult collect(const std::filesystem::path& root,
                                    const Options& options = Options{});

/// Collect statistics without materialising entries (scandir::Count).
///
/// `sorted` is ignored here, because Count has no such option.
[[nodiscard]] Statistics count(const std::filesystem::path& root,
                               const Options& options = Options{});

/// Group entries by type, per directory (scandir::Walk).
///
/// Note that scandir::Walk defaults skip_hidden to true while these options
/// default to false, matching Scandir and Count. Set it explicitly to
/// reproduce the Rust default.
[[nodiscard]] WalkResult walk(const std::filesystem::path& root,
                              const Options& options = Options{});

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

namespace detail {

/// Owns the `const char*` array the C API borrows for one call.
///
/// The C API borrows rather than copies filter patterns, so the array has to
/// outlive the call. Keeping it in a local (rather than caching it inside
/// Options) means Options stays immutable and safe to share across threads.
class FilterBinding {
  public:
    void bind(const std::vector<std::string>& patterns) {
        pointers_.clear();
        pointers_.reserve(patterns.size());
        for (const auto& pattern : patterns) {
            pointers_.push_back(pattern.c_str());
        }
        data_ = pointers_.empty() ? nullptr : pointers_.data();
        size_ = pointers_.size();
    }

    const char* const* data() const noexcept { return data_; }
    std::size_t size() const noexcept { return size_; }

  private:
    std::vector<const char*> pointers_;
    const char* const* data_ = nullptr;
    std::size_t size_ = 0;
};

/// RAII owner for a cscandir out-struct. Every handle here is initialised via
/// the matching cscandir_*_init call, which stamps the internal magic field
/// the C library uses to decide what it may free.
template <typename T, void (*Init)(T*), void (*Free)(T*)> class Handle {
  public:
    Handle() { Init(&value_); }
    ~Handle() { Free(&value_); }

    Handle(const Handle&) = delete;
    Handle& operator=(const Handle&) = delete;
    Handle(Handle&&) = delete;
    Handle& operator=(Handle&&) = delete;

    T* get() noexcept { return &value_; }

  private:
    T value_{};
};

using EntryListHandle =
    Handle<cscandir_entry_list, cscandir_entry_list_init, cscandir_free_entry_list>;
using StringListHandle =
    Handle<cscandir_string_list, cscandir_string_list_init, cscandir_free_string_list>;
using TocHandle = Handle<cscandir_toc, cscandir_toc_init, cscandir_free_toc>;
using WalkEntryListHandle = Handle<cscandir_walk_entry_list,
                                   cscandir_walk_entry_list_init,
                                   cscandir_free_walk_entry_list>;
using ErrorHandle = Handle<cscandir_error, cscandir_error_init, cscandir_free_error>;

/// Convert a path to the UTF-8 string the C API requires.
inline std::string to_utf8(const std::filesystem::path& path) {
#if defined(__cpp_lib_char8_t)
    const std::u8string utf8 = path.u8string();
    return std::string(reinterpret_cast<const char*>(utf8.data()), utf8.size());
#else
    return path.u8string();
#endif
}

inline std::vector<std::string> to_strings(const cscandir_string_list& list) {
    std::vector<std::string> out;
    out.reserve(list.len);
    for (std::size_t i = 0; i < list.len; ++i) {
        const char* item = list.items[i];
        out.emplace_back(item != nullptr ? item : "");
    }
    return out;
}

inline Toc to_toc(const cscandir_toc& src) {
    Toc toc;
    toc.dirs = to_strings(src.dirs);
    toc.files = to_strings(src.files);
    toc.symlinks = to_strings(src.symlinks);
    toc.other = to_strings(src.other);
    toc.errors = to_strings(src.errors);
    return toc;
}

inline Statistics to_statistics(const cscandir_statistics& src) {
    Statistics stats;
    stats.dirs = src.dirs;
    stats.files = src.files;
    stats.slinks = src.slinks;
    stats.hlinks = src.hlinks;
    stats.devices = src.devices;
    stats.pipes = src.pipes;
    stats.size = src.size;
    stats.usage = src.usage;
    stats.duration = src.duration;
    return stats;
}

/// Throw unless the call succeeded.
inline void check(std::int32_t code, const cscandir_error& error) {
    if (code == CSCANDIR_OK) {
        return;
    }
    const std::string message =
        error.message != nullptr ? std::string(error.message) : "unknown error";
    throw Error(static_cast<ErrorCode>(code), message);
}

}  // namespace detail

inline CollectResult collect(const std::filesystem::path& root,
                             const Options& options) {
    cscandir_options native = options.native();
    detail::FilterBinding dir_include, dir_exclude, file_include, file_exclude;
    dir_include.bind(options.dir_include());
    dir_exclude.bind(options.dir_exclude());
    file_include.bind(options.file_include());
    file_exclude.bind(options.file_exclude());
    native.dir_include = dir_include.data();
    native.dir_include_len = dir_include.size();
    native.dir_exclude = dir_exclude.data();
    native.dir_exclude_len = dir_exclude.size();
    native.file_include = file_include.data();
    native.file_include_len = file_include.size();
    native.file_exclude = file_exclude.data();
    native.file_exclude_len = file_exclude.size();

    const std::string root_utf8 = detail::to_utf8(root);

    detail::EntryListHandle entries;
    detail::StringListHandle errors;
    detail::ErrorHandle error;
    cscandir_statistics stats{};

    const std::int32_t code =
        cscandir_collect(root_utf8.c_str(), &native, entries.get(),
                         errors.get(), &stats, error.get());
    detail::check(code, *error.get());

    CollectResult result;
    result.entries.reserve(entries.get()->len);
    for (std::size_t i = 0; i < entries.get()->len; ++i) {
        const cscandir_entry& src = entries.get()->entries[i];
        Entry entry;
        entry.path = src.path != nullptr ? src.path : "";
        entry.is_symlink = src.is_symlink != 0;
        entry.is_dir = src.is_dir != 0;
        entry.is_file = src.is_file != 0;
        entry.ctime = src.ctime;
        entry.mtime = src.mtime;
        entry.atime = src.atime;
        entry.size = src.size;
        entry.has_ext = src.has_ext != 0;
        entry.mode = src.mode;
        entry.ino = src.ino;
        entry.dev = src.dev;
        entry.nlink = src.nlink;
        entry.blksize = src.blksize;
        entry.blocks = src.blocks;
        entry.uid = src.uid;
        entry.gid = src.gid;
        entry.rdev = src.rdev;
        result.entries.push_back(std::move(entry));
    }

    result.errors = detail::to_strings(*errors.get());
    result.stats = detail::to_statistics(stats);
    return result;
}

inline Statistics count(const std::filesystem::path& root,
                        const Options& options) {
    cscandir_options native = options.native();
    detail::FilterBinding dir_include, dir_exclude, file_include, file_exclude;
    dir_include.bind(options.dir_include());
    dir_exclude.bind(options.dir_exclude());
    file_include.bind(options.file_include());
    file_exclude.bind(options.file_exclude());
    native.dir_include = dir_include.data();
    native.dir_include_len = dir_include.size();
    native.dir_exclude = dir_exclude.data();
    native.dir_exclude_len = dir_exclude.size();
    native.file_include = file_include.data();
    native.file_include_len = file_include.size();
    native.file_exclude = file_exclude.data();
    native.file_exclude_len = file_exclude.size();

    const std::string root_utf8 = detail::to_utf8(root);

    detail::ErrorHandle error;
    cscandir_statistics stats{};

    const std::int32_t code =
        cscandir_count(root_utf8.c_str(), &native, &stats, nullptr,
                       error.get());
    detail::check(code, *error.get());
    return detail::to_statistics(stats);
}

inline WalkResult walk(const std::filesystem::path& root,
                       const Options& options) {
    cscandir_options native = options.native();
    detail::FilterBinding dir_include, dir_exclude, file_include, file_exclude;
    dir_include.bind(options.dir_include());
    dir_exclude.bind(options.dir_exclude());
    file_include.bind(options.file_include());
    file_exclude.bind(options.file_exclude());
    native.dir_include = dir_include.data();
    native.dir_include_len = dir_include.size();
    native.dir_exclude = dir_exclude.data();
    native.dir_exclude_len = dir_exclude.size();
    native.file_include = file_include.data();
    native.file_include_len = file_include.size();
    native.file_exclude = file_exclude.data();
    native.file_exclude_len = file_exclude.size();

    const std::string root_utf8 = detail::to_utf8(root);

    detail::TocHandle toc;
    detail::WalkEntryListHandle roots;
    detail::ErrorHandle error;
    cscandir_statistics stats{};

    const std::int32_t code =
        cscandir_walk(root_utf8.c_str(), &native, toc.get(), roots.get(),
                      &stats, error.get());
    detail::check(code, *error.get());

    WalkResult result;
    result.toc = detail::to_toc(*toc.get());
    result.roots.reserve(roots.get()->len);
    for (std::size_t i = 0; i < roots.get()->len; ++i) {
        const cscandir_walk_entry& src = roots.get()->entries[i];
        WalkEntry entry;
        entry.dir = src.path != nullptr ? src.path : "";
        entry.toc = detail::to_toc(src.toc);
        result.roots.push_back(std::move(entry));
    }
    result.stats = detail::to_statistics(stats);
    return result;
}

}  // namespace cscandir

#endif  // CSCANDIR_HPP
