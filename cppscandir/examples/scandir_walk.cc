// scandir_walk.cc - Example demonstrating the Walk function.
#include "cppscandir.h"
#include <iostream>
#include <cstdlib>

int main(int argc, char** argv) {
    const char* root = argc > 1 ? argv[1] : ".";

    cppscandir::Options options;
    options.skip_hidden = true;  // Match Rust Walk default
    options.sorted = true;

    try {
        cppscandir::WalkResult result = cppscandir::walk(root, options);

        std::cout << "Merged TOC: dirs=" << result.toc.dirs.size()
                  << " files=" << result.toc.files.size()
                  << " symlinks=" << result.toc.symlinks.size()
                  << " other=" << result.toc.other.size() << "\n";
        std::cout << "Directories visited: " << result.roots.size() << "\n";
        std::cout << "Statistics: dirs=" << result.statistics.dirs
                  << " files=" << result.statistics.files
                  << " duration=" << result.statistics.duration << "s\n\n";

        for (const auto& entry : result.roots) {
            std::cout << (entry.dir.empty() ? "." : entry.dir) << ": "
                      << "dirs=" << entry.toc.dirs.size()
                      << " files=" << entry.toc.files.size()
                      << " symlinks=" << entry.toc.symlinks.size()
                      << " other=" << entry.toc.other.size() << "\n";
        }

        if (!result.toc.errors.empty()) {
            std::cout << "\nScan errors:\n";
            for (const auto& err : result.toc.errors) {
                std::cout << "  " << err << "\n";
            }
        }

    } catch (const cppscandir::CppScandirError& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }

    return 0;
}