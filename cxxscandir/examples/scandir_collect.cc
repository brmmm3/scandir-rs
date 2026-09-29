// scandir_collect.cc - Example demonstrating the Collect function.
#include "cxxscandir.h"
#include <iostream>
#include <cstdlib>

int main(int argc, char** argv) {
    const char* root = argc > 1 ? argv[1] : ".";

    cxxscandir::Options options;
    options.sorted = true;
    options.skip_hidden = true;
    options.return_type = cxxscandir::ReturnType::Ext;

    auto result = cxxscandir::collect(root, options);
    if (!result.is_ok()) {
        std::cerr << "Error: " << result.error().message << " (code " << result.error().code << ")\n";
        return 1;
    }

    const auto& result_ok = result.ok();
    std::cout << "Found " << result_ok.entries.size() << " entries\n";
    std::cout << "Directories: " << result_ok.statistics.dirs
              << "  Files: " << result_ok.statistics.files
              << "  Symlinks: " << result_ok.statistics.slinks << "\n";
    std::cout << "Duration: " << result_ok.statistics.duration << "s\n\n";

    for (size_t i = 0; i < result_ok.entries.size() && i < 10; ++i) {
        const auto& entry = result_ok.entries[i];
        char kind = '?';
        if (entry.is_dir) kind = 'D';
        else if (entry.is_file) kind = 'F';
        else if (entry.is_symlink) kind = 'L';
        std::cout << "  " << kind << " " << entry.path << "\n";
        if (entry.has_ext) {
            std::cout << "    inode=" << entry.ino
                      << " mode=" << std::oct << entry.mode << std::dec
                      << " size=" << entry.size << "\n";
        }
    }

    if (!result_ok.errors.empty()) {
        std::cout << "\nScan errors:\n";
        for (const auto& err : result_ok.errors) {
            std::cout << "  " << err << "\n";
        }
    }

    return 0;
}