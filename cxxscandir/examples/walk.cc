#include <chrono>
#include <exception>
#include <iostream>
#include <string>

#include "cxxscandir/src/lib.rs.h"

int main(int argc, char** argv) {
  std::string root = "/usr";
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
  options.max_file_count = 100;
  options.extended = extended;

  try {
    const auto start = std::chrono::steady_clock::now();
    const auto result = cxxscandir::scan(options);
    const auto elapsed = std::chrono::duration<double>(
        std::chrono::steady_clock::now() - start);

    std::cout << "root=" << root << " entries=" << result.entries.size()
              << " errors=" << result.errors.size() << " duration="
              << elapsed.count() << " seconds\n";
    for (const auto& error : result.errors) {
      std::cerr << "scan error: " << static_cast<std::string>(error) << '\n';
    }
    const std::size_t displayed = result.entries.size() < 20 ? result.entries.size() : 20;
    for (std::size_t i = 0; i < displayed; ++i) {
      const auto& entry = result.entries[i];
      const char type = entry.is_symlink ? 'S' : entry.is_dir ? 'D' :
                        entry.is_file ? 'F' : '?';
      std::cout << type << ' ' << static_cast<std::string>(entry.path)
                << " size=" << entry.size << '\n';
    }
    if (result.entries.size() > displayed) {
      std::cout << "... (" << result.entries.size()
                << " entries total; showing first 20)\n";
    }
    return result.errors.empty() ? 0 : 1;
  } catch (const std::exception& error) {
    std::cerr << "walk failed: " << error.what() << '\n';
    return 1;
  }
}