#include "cscandir.hpp"

#include <algorithm>
#include <cstdio>
#include <iostream>
#include <string>

// File: scandir_collect.cpp
//
// Collect entries with the C++ wrapper. Results are ordinary C++ containers
// and are released automatically; nothing has to be freed by hand.
int main(int argc, char** argv) {
    const std::string root = argc > 1 ? argv[1] : ".";

    cscandir::Options options;
    options.sorted(true).return_type(cscandir::ReturnType::ext);

    try {
        cscandir::CollectResult result = cscandir::collect(root, options);

        std::cout << "entries: " << result.entries.size() << "\n";
        std::cout << "dirs: " << result.stats.dirs
                  << "  files: " << result.stats.files
                  << "  symlinks: " << result.stats.slinks << "\n";
        std::cout << "duration: " << result.stats.duration << "s\n";

        for (const std::string& error : result.errors) {
            std::cout << "  scan error: " << error << "\n";
        }

        std::cout << "\nfirst 10 entries:\n";
        for (std::size_t i = 0; i < result.entries.size() && i < 10; ++i) {
            const cscandir::Entry& entry = result.entries[i];
            const char kind = entry.is_dir      ? 'D'
                              : entry.is_file    ? 'F'
                              : entry.is_symlink ? 'L'
                                                 : '?';
            std::cout << "  " << kind << " " << entry.path << "\n";
            if (entry.has_ext) {
                std::cout << "      mode=" << std::oct << entry.mode
                          << std::dec << " size=" << entry.size
                          << " ino=" << entry.ino << "\n";
            }
        }

        // Group by top-level component for a quick summary.
        std::vector<std::string> top_level;
        for (const cscandir::Entry& entry : result.entries) {
            const std::string first = entry.path.substr(0, entry.path.find('/'));
            if (std::find(top_level.begin(), top_level.end(), first) ==
                top_level.end()) {
                top_level.push_back(first);
            }
        }
        std::cout << "\ndistinct top-level entries: " << top_level.size()
                  << "\n";
    } catch (const cscandir::Error& error) {
        std::fprintf(stderr, "cscandir failed (%d): %s\n",
                     static_cast<int>(error.code()), error.what());
        return 1;
    }

    return 0;
}
