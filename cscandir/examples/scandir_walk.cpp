#include "cscandir.hpp"

#include <cstdio>
#include <iostream>

// File: scandir_walk.cpp
//
// Group entries by directory using the C++ wrapper.
int main(int argc, char** argv) {
    const char* root = argc > 1 ? argv[1] : ".";

    try {
        // walk() defaults skip_hidden to false here, matching Scandir/Count.
        // Set it explicitly to reproduce the Rust Walk default.
        cscandir::Options options;
        options.skip_hidden(true);

        cscandir::WalkResult result = cscandir::walk(root, options);

        std::cout << "merged: dirs=" << result.toc.dirs.size()
                  << " files=" << result.toc.files.size()
                  << " symlinks=" << result.toc.symlinks.size()
                  << " other=" << result.toc.other.size() << "\n";

        for (const std::string& error : result.toc.errors) {
            std::cout << "  scan error: " << error << "\n";
        }

        std::cout << "directories visited: " << result.roots.size() << "\n";
        for (const cscandir::WalkEntry& entry : result.roots) {
            std::cout << "  " << (entry.dir.empty() ? "." : entry.dir) << ":"
                      << " dirs=" << entry.toc.dirs.size()
                      << " files=" << entry.toc.files.size() << "\n";
        }

        std::cout << "statistics: dirs=" << result.stats.dirs
                  << " files=" << result.stats.files << "\n";
    } catch (const cscandir::Error& error) {
        std::fprintf(stderr, "cscandir failed (%d): %s\n",
                     static_cast<int>(error.code()), error.what());
        return 1;
    }

    return 0;
}
