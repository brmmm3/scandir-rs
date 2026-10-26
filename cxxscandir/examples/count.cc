#include <exception>
#include <iostream>
#include <string>
#include <vector>

#include "cxxscandir/src/lib.rs.h"

int main(int argc, char** argv) {
  std::string root = "/tmp";
  bool have_root = false;
  bool extended = false;
  for (int i = 1; i < argc; ++i) {
    const std::string argument = argv[i];
    if (argument == "--ext") {
      extended = true;
    } else if (!have_root) {
      root = argument;
      have_root = true;
    } else {
      std::cerr << "Usage: " << argv[0] << " [ROOT] [--ext]\n";
      return 1;
    }
  }

  cxxscandir::ScanOptions options;
  options.root = rust::String(root);
  options.extended = extended;
  options.dir_exclude = {rust::String("dir0"), rust::String("dir1")};

  try {
    const auto stats = cxxscandir::count(options);
    std::cout << "root=" << root << " metadata="
              << (extended ? "extended" : "base") << '\n';
    std::cout << "dirs=" << stats.dirs << " files=" << stats.files
              << " symlinks=" << stats.symlinks << '\n';
    if (extended) {
      std::cout << "hardlinks=" << stats.hardlinks << " devices=" << stats.devices
                << " pipes=" << stats.pipes << '\n';
      std::cout << "size=" << stats.size << " usage=" << stats.usage << " bytes\n";
    }
    std::cout << "duration=" << stats.duration << " seconds\n";
    for (const auto& error : stats.errors) {
      std::cerr << "scan error: " << static_cast<std::string>(error) << '\n';
    }
    return stats.errors.empty() ? 0 : 1;
  } catch (const std::exception& error) {
    std::cerr << "count failed: " << error.what() << '\n';
    return 1;
  }
}