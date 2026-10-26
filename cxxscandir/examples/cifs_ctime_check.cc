#define _XOPEN_SOURCE 700

#include <cmath>
#include <cstdio>
#include <exception>
#include <iostream>
#include <string>

#include "cxxscandir/src/lib.rs.h"

#include <sys/stat.h>

namespace {

double stat_ctime_seconds(const struct stat& info) {
#if defined(__APPLE__)
  return static_cast<double>(info.st_ctimespec.tv_sec) +
         static_cast<double>(info.st_ctimespec.tv_nsec) * 1e-9;
#elif defined(_WIN32)
  return static_cast<double>(info.st_ctime);
#else
  return static_cast<double>(info.st_ctim.tv_sec) +
         static_cast<double>(info.st_ctim.tv_nsec) * 1e-9;
#endif
}

std::string join_path(const std::string& root, const std::string& relative) {
  if (root.empty() || root.back() == '/' || root.back() == '\\') {
    return root + relative;
  }
#ifdef _WIN32
  return root + "\\" + relative;
#else
  return root + "/" + relative;
#endif
}

}  // namespace

int main(int argc, char** argv) {
  if (argc > 2) {
    std::cerr << "Usage: " << argv[0] << " [MOUNT_PATH]\n";
    return 1;
  }
  const std::string root = argc == 2 ? argv[1] : "/mnt/smb";

  cxxscandir::ScanOptions options;
  options.root = rust::String(root);
  options.extended = true;

  try {
    const auto result = cxxscandir::scan(options);
    std::cout << "scanned " << result.entries.size() << " entries, "
              << result.errors.size() << " errors from " << root << '\n';
    std::size_t ctime_ok = 0;
    std::size_t ctime_none = 0;
    std::size_t stat_checks = 0;
    double max_ctime_skew = 0.0;
    for (const auto& entry : result.entries) {
      if (!entry.is_file) {
        continue;
      }
      const std::string relative = static_cast<std::string>(entry.path);
      if (entry.ctime > 0.0) {
        ++ctime_ok;
      } else {
        ++ctime_none;
        std::cout << "  CTIME-ZERO " << relative << " <<< REGRESSION?\n";
      }

      const std::string full_path = join_path(root, relative);
      struct stat info;
      bool has_stat_skew = false;
      double skew = 0.0;
      if (stat(full_path.c_str(), &info) == 0) {
        skew = std::fabs(entry.ctime - stat_ctime_seconds(info));
        if (skew > max_ctime_skew) {
          max_ctime_skew = skew;
        }
        ++stat_checks;
        has_stat_skew = true;
      }

      std::cout << "  " << relative << " ctime=" << entry.ctime;
      std::cout << " mode=" << entry.mode << " ino=" << entry.inode;
      if (has_stat_skew) {
        std::cout << " stat-skew=" << skew << "s";
      }
      std::cout << '\n';
    }
    for (const auto& error : result.errors) {
      std::cerr << "scan error: " << static_cast<std::string>(error) << '\n';
    }
    std::cout << "summary: ctime " << ctime_ok << " nonzero / " << ctime_none
              << " zero, stat checks=" << stat_checks
              << ", max ctime skew=" << max_ctime_skew << "s\n";
    if (stat_checks == 0) {
      std::cerr << "warning: no entries could be checked against stat()\n";
    }
    return ctime_none != 0 || !result.errors.empty() ? 1 : 0;
  } catch (const std::exception& error) {
    std::cerr << "scan failed: " << error.what() << '\n';
    return 1;
  }
}