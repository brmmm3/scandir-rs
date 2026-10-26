#include <exception>
#include <iostream>
#include <string>

#include "cxxscandir/src/lib.rs.h"

int main(int argc, char** argv) {
  cxxscandir::ScanOptions options;
  options.root = rust::String(argc > 1 ? argv[1] : ".");
  options.sorted = true;
  options.extended = true;
  options.max_file_count = 10;

  try {
    const auto result = cxxscandir::scan(options);
    std::cout << "entries=" << result.entries.size()
              << " errors=" << result.errors.size()
              << " duration=" << result.duration << "s\n";
    for (const auto& entry : result.entries) {
      std::cout << (entry.is_dir ? "[dir] " : entry.is_symlink ? "[link] " : "[file] ")
                << static_cast<std::string>(entry.path) << " (" << entry.size
                << " bytes)\n";
    }
    for (const auto& error : result.errors) {
      std::cerr << "scan warning: " << static_cast<std::string>(error) << '\n';
    }

    const auto stats = cxxscandir::count(options);
    std::cout << "files=" << stats.files << " hardlinks=" << stats.hardlinks
              << " size=" << stats.size << " usage=" << stats.usage << '\n';
  } catch (const std::exception& error) {
    std::cerr << "scan failed: " << error.what() << '\n';
    return 1;
  }
  return 0;
}