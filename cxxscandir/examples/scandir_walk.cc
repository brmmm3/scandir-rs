// scandir_walk.cc - Example demonstrating the Walk function.
#include "cxxscandir.h"
#include <iostream>
#include <cstdlib>

int main(int argc, char** argv) {
    const char* root = argc > 1 ? argv[1] : ".";

    cxxscandir::Options options;
    options.skip_hidden = true;  // Match Rust Walk default
    options.sorted = true;

    auto result = cxxscandir::walk(root, options);
    if (!result.is_ok()) {
        std::cerr << "Error: " << result.error().message << " (code " << result.error().code << ")\n";
        return 1;
    }

    const auto& result_ok = result.ok();
    std::cout << "Merged TOC: dirs=" << result_ok.toc.dirs.size()
              << " files=" << result_ok.toc.files.size()
              << " symlinks=" << result_ok.toc.symlinks.size()
              << " other=" << result_ok.toc.other.size() << "\n";
    std::cout << "Directories visited: " << result_ok.roots.size() << "\n";
    std::cout << "Statistics: dirs=" << result_ok.statistics.dirs
              << " files=" << result_ok.statistics.files
              << " duration=" << result_ok.statistics.duration << "s\n\n";

    for (const auto& entry : result_ok.roots) {
        std::cout << (entry.path.empty() ? "." : entry.path) << ": "
                  << "dirs=" << entry.toc.dirs.size()
                  << " files=" << entry.toc.files.size()
                  << " symlinks=" << entry.toc.symlinks.size() << "\n";
    }

    if (!result_ok.toc.errors.empty()) {
        std::cout << "\nScan errors:\n";
        for (const auto& err : result_ok.toc.errors) {
            std::cout << "  " << err << "\n";
        }
    }

    return 0;
}