---
title: C++
---

The `cxxscandir` crate exposes C++-usable structures through the Rust
[`cxx`](https://cxx.rs/) bridge. It calls the Rust `Scandir` and `Count`
implementations directly rather than going through the C ABI.

## Build the examples and library

From the repository root:

```sh
cargo build -p cxxscandir --release
make -C cxxscandir build
make -C cxxscandir run ROOT=/var/log
```

The Makefile builds the Rust static library, locates the generated bridge
headers, and links the example programs. It expects a C++ compiler and, on
Linux, links the standard C++ runtime plus pthread, dl, and math libraries.

## Scan and count

```cpp
#include <exception>
#include <iostream>
#include <string>

#include "cxxscandir/src/lib.rs.h"

int main(int argc, char** argv) {
  cxxscandir::ScanOptions options;
  options.root = rust::String(argc > 1 ? argv[1] : "/var/log");
  options.sorted = true;
  options.extended = true;
  options.max_depth = 3;

  try {
    const auto result = cxxscandir::scan(options);
    for (const auto& entry : result.entries) {
      std::cout << static_cast<std::string>(entry.path) << '\n';
    }
    for (const auto& warning : result.errors) {
      std::cerr << static_cast<std::string>(warning) << '\n';
    }

    const auto stats = cxxscandir::count(options);
    std::cout << "files=" << stats.files << " size=" << stats.size << '\n';
  } catch (const std::exception& error) {
    std::cerr << "scan failed: " << error.what() << '\n';
    return 1;
  }
}
```

`ScanOptions` includes sorting, hidden-entry handling, depth/file limits,
directory/file glob filters, case sensitivity, link following, and extended
metadata. Zero limits mean unlimited. `scan()` returns entries and nonfatal
traversal errors; fatal errors throw `std::exception`. `count()` avoids
materializing entries. Descendant entry paths are relative to the root, and
the root entry comes first.

## Integrate with another C++ build

Use the generated `cxxscandir/src/lib.rs.h` header and the static library
produced by Cargo. Add Cargo's generated
`target/<profile>/build/cxxscandir-*/out/cxxbridge/include` directory to the
header search path and link `libcxxscandir.a` plus the C++ runtime and native
system libraries required by your platform. The in-repository Makefile is the
working reference for locating generated headers and linking the library; see
the [C++ README](https://github.com/brmmm3/scandir-rs/blob/master/cxxscandir/README.md).
