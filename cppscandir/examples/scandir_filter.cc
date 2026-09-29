// scandir_filter.cc - Example demonstrating filter options.
#include "cppscandir.h"
#include <iostream>
#include <cstdlib>
#include <algorithm>

int main(int argc, char** argv) {
    const char* root = argc > 1 ? argv[1] : ".";

    cppscandir::Options options;
    options.file_include = {"*.cpp", "*.h", "*.hpp"};
    options.dir_exclude = {"build", "target", ".git", "node_modules"};
    options.sorted = true;
    options.return_type = cppscandir::ReturnType::Ext;

    try {
        std::cout << "Filtering: " << root << "\n";
        std::cout << "  File patterns: *.cpp, *.h, *.hpp\n";
        std::cout << "  Excluded dirs: build, target, .git, node_modules\n\n";

        cppscandir::CollectResult result = cppscandir::collect(root, options);

        std::cout << "Found " << result.entries.size() << " entries\n";
        std::cout << "Directories: " << result.statistics.dirs
                  << "  Files: " << result.statistics.files << "\n";

        int fileCount = 0;
        for (const auto& entry : result.entries) {
            if (entry.is_file) {
                ++fileCount;
                if (fileCount <= 20) {
                    std::cout << "  " << entry.path << "\n";
                } else if (fileCount == 21) {
                    std::cout << "  ... and " << (result.entries.size() - 20) << " more\n";
                    break;
                }
            }
        }
        std::cout << "Total matching files: " << fileCount << "\n";

    } catch (const cppscandir::CppScandirError& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }

    return 0;
}