# cscandir

C ABI wrapper for the `scandir` crate.

## Build

```bash
cargo build -p cscandir --release
```

Artifacts are generated in `target/release/`:

- `cscandir.dll` / `libcscandir.so` / `libcscandir.dylib`
- `cscandir.lib` / `libcscandir.a`

Public header is in `include/cscandir.h`.

## API notes

- Call `cscandir_options_init()` before customizing options, or pass `NULL`
  for defaults. Boolean options accept zero or nonzero; zero and `SIZE_MAX`
  mean unlimited for depth and file limits. Directory globs are root-relative;
  file globs match names. Filter strings must remain readable during the call.
- `cscandir_collect()` returns entries with basic or extended metadata.
  Descendant paths are relative to the root. The first entry is the root itself,
  named with its basename, and may lack stat metadata. Sorted mode preserves
  the core's depth-first traversal with sorted siblings, not global path sorting.
- `cscandir_count()` returns aggregate statistics, excluding the root directory.
  Set `return_type = CSCANDIR_RETURN_EXT` for size, disk usage and extended
  file-type accounting; `sorted` has no effect on count.
- Inputs must be valid NUL-terminated UTF-8 strings. A nonzero filter length
  requires a non-NULL array containing no NULL elements. Invalid options and
  glob syntax return `CSCANDIR_ERR_INVALID_ARGUMENT`.
- Every output pointer is optional. Outputs may initially be uninitialized;
  the API only writes them. **Free previous outputs before reusing their storage**:
  scan calls overwrite output structs and never free their old contents.
- On failure, lists/statistics are empty/zero and `out_error`, if requested,
  contains the error code and message. On success it is `{CSCANDIR_OK, NULL}`.
  Per-entry traversal errors are separate from fatal call errors: a call can
  return `CSCANDIR_OK` with nonempty `out_errors`.
- Free buffers only with `cscandir_free_entry_list()`,
  `cscandir_free_string_list()` and `cscandir_free_error()`. They reset their
  structs and are NULL-safe and repeatable on the same storage. Never free
  individual items, change buffer lengths/capacities, or free shallow copies.
- Rust panics are caught at scan entry points and reported as
  `CSCANDIR_ERR_PANIC` with default unwinding builds. Invalid pointers,
  allocation aborts and builds with `panic = "abort"` cannot be recovered.
  Struct layouts and existing exported C signatures are unchanged.

```c
#include <stdio.h>
#include "cscandir.h"

int main(int argc, char **argv) {
    cscandir_options options;
    cscandir_options_init(&options);
    options.sorted = 1;
    cscandir_entry_list entries = {0};
    cscandir_string_list errors = {0};
    cscandir_error error = {0};

    int32_t status = cscandir_collect(argc > 1 ? argv[1] : ".", &options,
                                      &entries, &errors, &error);
    if (status == CSCANDIR_OK) {
        for (size_t i = 0; i < entries.len; ++i)
            puts(entries.entries[i].path);
        for (size_t i = 0; i < errors.len; ++i)
            fprintf(stderr, "%s\n", errors.items[i]);
    } else {
        fprintf(stderr, "%s\n", error.message ? error.message : "scan failed");
    }

    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
    return status == CSCANDIR_OK ? 0 : 1;
}
```

## C counterparts for the Rust examples

The four runnable examples in `scandir/examples/` have C counterparts in
`cscandir/examples/`:

| Rust example | C example | Purpose |
| --- | --- | --- |
| `count.rs` | `count.c` | Aggregate statistics, directory exclusions, and optional extended accounting |
| `scandir.rs` | `scandir.c` | Sorted entry collection with optional extended metadata |
| `walk.rs` | `walk.c` | Entry collection capped at 100 files, with optional extended metadata |
| `cifs_ctime_check.rs` | `cifs_ctime_check.c` | CIFS timestamp check plus a `stat()` ctime cross-check |

`benchmark.c` is an additional C API microbenchmark. C calls are synchronous:
the ABI does not expose the Rust examples' background `start()`/`stop()` controls
or progress polling. `cifs_ctime_check.c` checks `ctime` because the current C
entry ABI does not expose the Rust `st_btime` birth-time field.

Build all examples and run individual programs from the repository root:

```bash
make -C cscandir build
make -C cscandir run-count ROOT=/tmp EXT=1
make -C cscandir run-scandir ROOT=/usr EXT=1
make -C cscandir run-walk ROOT=/usr EXT=1
make -C cscandir run-cifs ROOT=/mnt/smb
```

`run` remains a shorthand for `run-scandir`. `EXT=1` enables extended metadata
for the count, scandir and walk examples.

## Validation

From the repository root, using the Linux/POSIX Makefile helpers:

```bash
make -C cscandir build             # release library and all C examples
make -C cscandir run ROOT=/usr     # run the scandir example
make -C cscandir build-debug       # debug library and all examples
make -C cscandir test              # Rust tests and C ABI regressions
make -C cscandir test-asan         # C AddressSanitizer/UBSan and leak checks
cargo clippy -p cscandir --all-targets -- -D warnings
```

The C tests cover poisoned/uninitialized output storage, invalid arguments and
UTF-8, filters, sorted/extended entries, count, omitted outputs and repeated frees.
Rust tests cover panic containment and rollback of partially converted outputs.
The sanitizer target instruments the C test harness, not the Rust library;
it does not replace Rust-side memory-safety analysis. The Makefile embeds the
library directory as a development runtime search path. Override `CC`, `CARGO`,
`TARGET_DIR`, `CFLAGS` or `LDFLAGS` as needed; the C API itself is not Linux-only.

## Performance

The wrapper consumes owned Rust results, moves path buffers into C strings,
avoids cloning the result set and skips C conversions for omitted outputs.
Partial conversions are cleaned up on errors. Extended scans also reuse the
already-built relative path instead of constructing it twice in the core.

```bash
make -C cscandir benchmark ROOT=/usr ITERATIONS=50
```

The benchmark reports collect, collect without outputs, count and extended
collect, with a warm-up per mode. Timings include conversion and deallocation.
Use a fixed directory tree, optimized builds and multiple alternating
before/after rounds; filesystem cache state and system load affect results.
Omitting `out_entries` skips conversion, not the scan or core result storage.
For aggregate-only workloads, prefer `cscandir_count()` to collecting entries.

Local Linux release-build measurements on a fixed tree of 20,000 files and
100 subdirectories (20,101 collected entries including the root), with warm
filesystem caches, gave the following medians across five alternating
before/after rounds of 100 scans per mode:

| Mode | Before (ms/scan) | After (ms/scan) | Elapsed-time reduction |
| --- | ---: | ---: | ---: |
| Collect | 11.941 | 10.842 | 9.2% |
| Collect without outputs | 11.737 | 9.017 | 23.2% |
| Count | 8.614 | 8.578 | 0.4% (effectively unchanged) |
| Extended collect | 11.253 | 10.504 | 6.7% |

These are workload-specific observations, not guaranteed speedups. The
no-output mode still traverses and stores results in the core; it avoids only
the wrapper's conversion work. Re-run the benchmark on representative data
before drawing conclusions about an application's performance.
