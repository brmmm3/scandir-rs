/**
 * cppscandir - Traditional C++ FFI wrapper for scandir-rs
 * 
 * This is a traditional C++ FFI wrapper (not using cxx crate) that directly
 * calls the C API from cscandir.h
 */

#ifndef CPPSCANDIR_H
#define CPPSCANDIR_H

#include <cstdint>
#include <string>
#include <vector>
#include <variant>
#include <optional>
#include <stdexcept>

namespace cppscandir {

// Error codes from the C library
enum class ErrorCode : int32_t {
    Ok = 0,
    InvalidArgument = 1,
    InvalidUtf8 = 2,
    Scan = 3,
    NulByte = 4
};

enum class ReturnType : uint32_t {
    Base = 0,
    Ext = 1
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
    std::string dir;
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

class CppScandirError : public std::runtime_error {
public:
    explicit CppScandirError(int32_t code, const std::string& msg)
        : std::runtime_error("cppscandir error " + std::to_string(code) + ": " + msg),
          code_(code) {}

    int32_t code() const noexcept { return code_; }

private:
    int32_t code_;
};

// Main API functions
CollectResult collect(const std::string& root, const Options& options = {});
Statistics count(const std::string& root, const Options& options = {});
WalkResult walk(const std::string& root, const Options& options = {});

} // namespace cppscandir

#endif // CPPSCANDIR_H