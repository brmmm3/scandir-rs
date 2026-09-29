// cxxscandir.h - C++ header for the cxxscandir wrapper
#ifndef CXXSCANDIR_H
#define CXXSCANDIR_H

#include <cstdint>
#include <string>
#include <vector>
#include <variant>
#include <optional>

namespace cxxscandir {

enum class ErrorCode : int32_t {
    Ok = 0,
    InvalidArgument = 1,
    InvalidUtf8 = 2,
    Scan = 3,
    NulByte = 4,
};

enum class ReturnType : uint32_t {
    Base = 0,
    Ext = 1,
};

struct Statistics {
    int32_t dirs = 0;
    int32_t files = 0;
    int32_t slinks = 0;
    int32_t hlinks = 0;
    int32_t devices = 0;
    int32_t pipes = 0;
    uint64_t size = 0;
    uint64_t usage = 0;
    double duration = 0.0;
};

struct Entry {
    std::string path;
    bool is_symlink = false;
    bool is_dir = false;
    bool is_file = false;
    double ctime = 0.0;
    double mtime = 0.0;
    double atime = 0.0;
    uint64_t size = 0;
    bool has_ext = false;
    uint32_t mode = 0;
    uint64_t ino = 0;
    uint64_t dev = 0;
    uint64_t nlink = 0;
    uint64_t blksize = 0;
    uint64_t blocks = 0;
    uint32_t uid = 0;
    uint32_t gid = 0;
    uint64_t rdev = 0;
};

struct Toc {
    std::vector<std::string> dirs;
    std::vector<std::string> files;
    std::vector<std::string> symlinks;
    std::vector<std::string> other;
    std::vector<std::string> errors;

    bool empty() const noexcept {
        return dirs.empty() && files.empty() && symlinks.empty() &&
               other.empty() && errors.empty();
    }
};

struct WalkEntry {
    std::string path;
    Toc toc;
};

struct Options {
    bool sorted = false;
    bool skip_hidden = false;
    uint64_t max_depth = 0;
    uint64_t max_file_cnt = 0;
    std::vector<std::string> dir_include;
    std::vector<std::string> dir_exclude;
    std::vector<std::string> file_include;
    std::vector<std::string> file_exclude;
    bool case_sensitive = false;
    bool follow_links = false;
    ReturnType return_type = ReturnType::Base;
};

struct Error {
    int32_t code = 0;
    std::string message;
};

template<typename T>
class Result {
public:
    Result() : has_value_(false), error_(Error{}) {}
    Result(T value) : has_value_(true), value_(std::move(value)), error_(Error{}) {}
    Result(Error error) : has_value_(false), error_(std::move(error)) {}

    bool is_ok() const noexcept { return has_value_; }
    bool is_err() const noexcept { return !has_value_; }

    const T& ok() const {
        return value_;
    }
    T& ok() {
        return value_;
    }
    const Error& error() const noexcept {
        return error_;
    }
    Error& error() noexcept {
        return error_;
    }

private:
    bool has_value_;
    T value_;
    Error error_;
};

struct CollectResult {
    std::vector<Entry> entries;
    std::vector<std::string> errors;
    Statistics statistics;
};

struct WalkResult {
    Toc toc;
    std::vector<WalkEntry> roots;
    Statistics statistics;
};

// Returns default options (matching scandir::Scandir defaults)
Options options_default();

// Main entry points
Result<CollectResult> collect(const std::string& root, const Options& options = {});
Result<Statistics> count(const std::string& root, const Options& options = {});
Result<WalkResult> walk(const std::string& root, const Options& options = {});

}  // namespace cxxscandir

#endif  // CXXSCANDIR_H