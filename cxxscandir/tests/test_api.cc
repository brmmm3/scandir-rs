#define _XOPEN_SOURCE 700

#include <cstdio>
#include <cstdlib>
#include <exception>
#include <string>

#ifdef _WIN32
#include <windows.h>
#else
#include <sys/stat.h>
#include <unistd.h>
#endif

#include "cxxscandir/src/lib.rs.h"

namespace {

void ensure(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "test failure: %s\n", message);
    std::abort();
  }
}

std::string join_path(const std::string& root, const std::string& child) {
#ifdef _WIN32
  const char separator = '\\';
#else
  const char separator = '/';
#endif
  if (!root.empty() && (root.back() == '/' || root.back() == '\\')) {
    return root + child;
  }
  return root + separator + child;
}

class TemporaryDirectory {
 public:
  TemporaryDirectory() {
#ifdef _WIN32
    char temp_path[MAX_PATH];
    char unique_path[MAX_PATH];
    ensure(GetTempPathA(MAX_PATH, temp_path) != 0, "get temporary directory");
    ensure(GetTempFileNameA(temp_path, "cxx", 0, unique_path) != 0,
           "create unique temporary path");
    ensure(DeleteFileA(unique_path) != 0, "remove temporary placeholder");
    ensure(CreateDirectoryA(unique_path, nullptr) != 0,
           "create temporary directory");
    path_ = unique_path;
#else
    char path[] = "/tmp/cxxscandir-test-XXXXXX";
    char* created = mkdtemp(path);
    ensure(created != nullptr, "create temporary directory");
    path_ = created;
#endif
  }

  ~TemporaryDirectory() {
    const std::string subdir = join_path(path_, "subdir");
#ifdef _WIN32
    DeleteFileA(join_path(subdir, "data.txt").c_str());
    DeleteFileA(join_path(subdir, "data-link.txt").c_str());
    RemoveDirectoryA(subdir.c_str());
    RemoveDirectoryA(path_.c_str());
#else
    std::remove(join_path(subdir, "data.txt").c_str());
    std::remove(join_path(subdir, "data-link.txt").c_str());
    rmdir(subdir.c_str());
    rmdir(path_.c_str());
#endif
  }

  const std::string& path() const { return path_; }

 private:
  std::string path_;
};

void create_fixture(const std::string& root) {
  const std::string subdir = join_path(root, "subdir");
#ifdef _WIN32
  ensure(CreateDirectoryA(subdir.c_str(), nullptr) != 0, "create subdirectory");
#else
  ensure(mkdir(subdir.c_str(), 0700) == 0, "create subdirectory");
#endif

  const std::string file_path = join_path(subdir, "data.txt");
  FILE* file = std::fopen(file_path.c_str(), "wb");
  ensure(file != nullptr, "create fixture file");
  ensure(std::fwrite("hello", 1, 5, file) == 5, "write fixture file");
  ensure(std::fclose(file) == 0, "close fixture file");
#ifdef _WIN32
  ensure(CreateHardLinkA(join_path(subdir, "data-link.txt").c_str(),
                         file_path.c_str(), nullptr) != 0,
         "create hard link");
#else
  ensure(link(file_path.c_str(), join_path(subdir, "data-link.txt").c_str()) == 0,
         "create hard link");
#endif
}

cxxscandir::ScanOptions options_for(const std::string& root) {
  cxxscandir::ScanOptions options;
  options.root = rust::String(root);
  options.sorted = true;
  options.skip_hidden = false;
  options.extended = true;
  return options;
}

void test_scan_and_count_with_extended_metadata() {
  TemporaryDirectory root;
  create_fixture(root.path());

  const auto options = options_for(root.path());
  const auto result = cxxscandir::scan(options);
  ensure(result.errors.empty(), "scan has no traversal errors");
  ensure(result.entries.size() == 4, "scan includes root, directory, and both links");
  ensure(result.duration >= 0.0, "scan duration is nonnegative");

  bool found_file = false;
  for (const auto& entry : result.entries) {
    const std::string path = static_cast<std::string>(entry.path);
    if (path.size() >= std::string("data.txt").size() &&
        path.compare(path.size() - std::string("data.txt").size(),
                     std::string("data.txt").size(), "data.txt") == 0) {
      ensure(entry.is_file, "fixture entry is a file");
      ensure(entry.size == 5, "fixture file size is preserved");
      ensure(entry.inode > 0, "extended inode metadata is populated");
      found_file = true;
    }
  }
  ensure(found_file, "scan returns fixture file");

  const auto stats = cxxscandir::count(options);
  ensure(stats.dirs == 1, "count finds the subdirectory");
  ensure(stats.files == 1, "count finds the unique file");
  ensure(stats.hardlinks == 1, "count identifies the hard link");
  ensure(stats.size >= 10, "count accounts for both hard-link paths");
  ensure(stats.duration >= 0.0, "count duration is nonnegative");
  ensure(stats.errors.empty(), "count has no traversal errors");
}

void test_invalid_root_returns_error() {
  cxxscandir::ScanOptions options;
#ifdef _WIN32
  options.root = rust::String("Z:\\cxxscandir-invalid-root-test");
#else
  options.root = rust::String("/path/that/does/not/exist/cxxscandir-test");
#endif

  bool threw = false;
  try {
    (void)cxxscandir::scan(options);
  } catch (const std::exception&) {
    threw = true;
  }
  ensure(threw, "invalid root raises an exception");
}

}  // namespace

int main() {
  test_scan_and_count_with_extended_metadata();
  test_invalid_root_returns_error();
  std::puts("cxxscandir C++ tests passed");
  return 0;
}