# cppscandir

Traditional C++ FFI wrapper for the `scandir-rs` directory scanner.

Uses traditional C++ FFI (not cxx crate) for C interop. C++17 compatible.

## Usage

```cpp
#include "cppscandir.h"

cppscandir::Options options;
options.sorted = true;
options.skip_hidden = true;
options.return_type = cppscandir::ReturnType::Ext;

cppscandir::CollectResult result = cppscandir::collect("/usr", options);

for (const auto& entry : result.entries) {
    std::cout << entry.path << " (" << (entry.is_dir ? "DIR" : "FILE") << ")\n";
}
```

## Building

```bash
# Build libcscandir first
cd ../cscandir && cargo build --release

# Build cppscandir
cmake -B build
cmake --build build
```

## Testing

```bash
cd build && ctest
```

## Requirements

- C++17 compiler
- `libcscandir.so` (or `cscandir.dll` on Windows)
  - Set `CSCANDIR_LIB_PATH` or place in `../target/release`

## API

| Function | Purpose |
| --- | --- |
| `collect(root, options)` | Entries with type and stat info |
| `count(root, options)` | Aggregate statistics only |
| `walk(root, options)` | Entries grouped by type, per-directory |

## Options

```cpp
cppscandir::Options options;
options.sorted = true;
options.skip_hidden = true;
options.max_depth = 10;
options.max_file_cnt = 1000;
options.dir_include = {"src", "include"};
options.dir_exclude = {"target", "build"};
options.file_include = {"*.cpp", "*.h"};
options.file_exclude = {"*.o"};
options.case_sensitive = true;
options.follow_links = false;
options.return_type = cppscandir::ReturnType::Ext;
```

## Error Handling

Functions throw `cppscandir::CppScandirError` (derives from `std::runtime_error`).
Per-path scan errors are returned in `result.errors`.

## License

MIT OR Apache-2.0