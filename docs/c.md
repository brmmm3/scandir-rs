---
title: C
---

The `cscandir` crate exports a C ABI with synchronous `cscandir_collect()` and
`cscandir_count()` calls. The public header is
[`cscandir.h`](https://github.com/brmmm3/scandir-rs/blob/master/cscandir/include/cscandir.h).

## Build and link

From a repository checkout:

```sh
cargo build -p cscandir --release
```

The library is written to `target/release/`. A Linux example compile command
from the repository root is:

```sh
cc -Icscandir/include example.c -Ltarget/release \
  -Wl,-rpath,"$PWD/target/release" -lcscandir -o example
```

Use the platform's compiler, library filename, and runtime search-path options
on macOS or Windows. In a deployed application, install the header and native
library in locations appropriate to your build and packaging system.

## Collect entries

```c
#include <stdio.h>
#include "cscandir.h"

int main(void) {
    cscandir_options options;
    cscandir_options_init(&options);
    options.sorted = 1;
    options.return_type = CSCANDIR_RETURN_EXT;

    cscandir_entry_list entries = {0};
    cscandir_string_list traversal_errors = {0};
    cscandir_error error = {0};
    int32_t status = cscandir_collect("/var/log", &options, &entries,
                                      &traversal_errors, &error);
    if (status != CSCANDIR_OK) {
        fprintf(stderr, "scan failed: %s\n",
                error.message ? error.message : "unknown error");
    } else {
        for (size_t i = 0; i < entries.len; ++i) {
            puts(entries.entries[i].path);
        }
        for (size_t i = 0; i < traversal_errors.len; ++i) {
            fprintf(stderr, "warning: %s\n", traversal_errors.items[i]);
        }
    }

    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&traversal_errors);
    cscandir_free_error(&error);
    return status == CSCANDIR_OK ? 0 : 1;
}
```

Initialize options before changing fields, or pass `NULL` for native defaults.
Every output is optional. A fatal error is returned as a nonzero status and
described by `out_error`; individual traversal errors may be returned in
`out_errors` even when status is `CSCANDIR_OK`.

## Count without materializing entries

Use `cscandir_count(root, options, &statistics, &errors, &error)` when you only
need aggregate statistics. Set `options.return_type` to
`CSCANDIR_RETURN_EXT` for extended accounting such as size, usage, hardlinks,
and device/file-type counts. Count excludes the root directory; `collect`
includes a synthetic root entry.

## Ownership rules

All output buffers are owned by the native library. Free them only with the
matching `cscandir_free_*` functions, and free existing output contents before
reusing their storage. Never free individual elements with `free()`, alter
length/capacity fields, or free a shallow copy of an output. Inputs must be
valid NUL-terminated UTF-8 strings and remain readable for the duration of the
call. See the [C binding README](https://github.com/brmmm3/scandir-rs/blob/master/cscandir/README.md)
for complete ABI and build details.
