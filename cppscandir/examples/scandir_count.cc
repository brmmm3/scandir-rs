// scandir_count.cc - Example demonstrating the Count function.
#include "cppscandir.h"
#include <iostream>
#include <cstdlib>

int main(int argc, char** argv) {
    const char* root = argc > 1 ? argv[1] : ".";

    cppscandir::Options options;
    options.skip_hidden = true;

    try {
        cppscandir::Statistics stats = cppscandir::count(root, options);

        std::cout << "Statistics:\n";
        std::cout << "  Directories: " << stats.dirs << "\n";
        std::cout << "  Files: " << stats.files << "\n";
        std::cout << "  Symlinks: " << stats.slinks << "\n";
        std::cout << "  Hardlinks: " << stats.hlinks << "\n";
        std::cout << "  Devices: " << stats.devices << "\n";
        std::cout << "  Pipes: " << stats.pipes << "\n";
        std::cout << "  Total size: " << stats.size << " bytes\n";
        std::cout << "  Disk usage: " << stats.usage << " bytes\n";
        std::cout << "  Duration: " << stats.duration << "s\n";

    } catch (const cppscandir::CppScandirError& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }

    return 0;
}