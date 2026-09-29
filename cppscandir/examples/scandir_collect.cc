// scandir_collect.cc - Example demonstrating the Collect function.
#include "cppscandir.h"
#include <iostream>
#include <cstdlib>

int main(int argc, char** argv) {
    const char* root = argc > 1 ? argv[1] : ".";

    cppscandir::Options options;
    options.sorted = true;
    options.skip_hidden = true;
    options.return_type = cppscandir::ReturnType::Ext;

    try {
        cppscandir::CollectResult result = cppscandir::collect(root, options);

        std::cout << "Found " << result.entries.size() << " entries\n";
        std::cout << "Directories: " << result.statistics.dirs
                  << "  Files: " << result.statistics.files
                  << "  Symlinks: " << result.statistics.slinks << "\n";
        std::cout << "Duration: " << result.statistics.duration << "s\n\n";

        std::cout << "First 10 entries:\n";
        for (size_t i = 0; i < result.entries.size() && i < 10; ++i) {
            const auto& entry = result.entries[i];
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

        if (!result.errors.empty()) {
            std::cout << "\nScan errors:\n";
            for (const auto& err : result.errors) {
                std::cout << "  " << err << "\n";
            }
        }

    } catch (const cppscandir::CppScandirError& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }

    return 0;
}