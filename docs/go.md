---
title: Go
---

The Go module in `goscandir/` wraps the `cscandir` C ABI using cgo. It requires
cgo, a C compiler, the C header, and a built `libcscandir` shared library.
Collected native values are copied into Go-owned values before the C buffers
are freed.

## Build the native dependency

From a repository checkout:

```sh
cargo build -p cscandir --release
cd goscandir
go test ./...
```

The module's default cgo flags expect the header in `../cscandir/include` and
the native library in `../target/release`. If you install the header/library
elsewhere, provide the appropriate `CGO_CFLAGS` and `CGO_LDFLAGS` for your
toolchain. For example:

```sh
CGO_CFLAGS="-I/path/to/include" \
CGO_LDFLAGS="-L/path/to/lib -Wl,-rpath,/path/to/lib" \
  go test ./...
```

The native library must also be discoverable when the resulting program runs.
The exact linker and runtime search-path flags are platform-specific. The
repository module is easiest to build from the checkout; when consuming it as a
separate Go dependency, ensure its cgo header and native-library prerequisites
are installed and configured in your application build.

## Collect entries

```go
package main

import (
	"fmt"
	"log"

	"github.com/brmmm3/scandir-rs/goscandir"
)

func main() {
	options := &cscandir.Options{
		Sorted:     true,
		SkipHidden: true,
		FileInclude: []string{"*.log"},
	}
	result, err := cscandir.Collect("/var/log", options)
	if err != nil {
		log.Fatal(err)
	}
	for _, entry := range result.Entries {
		fmt.Println(entry.Path)
	}
	for _, warning := range result.Errors {
		fmt.Println("warning:", warning)
	}
}
```

The actual package name is `cscandir`. Descendant paths are relative to the
root, and `Collect` includes a synthetic root entry. Use `Count(root, options)`
for aggregate values without materializing entries. Extended counts require
`options.ReturnType = cscandir.ReturnTypeExt`.

## Walk and options

`Walk(root, options)` groups a complete `Collect` result by parent directory;
it uses the same memory for entries as `Collect`, so it is not streaming. Options
include sorted order, hidden handling, maximum depth/count, directory/file glob
filters, case sensitivity, link following, and basic/extended return type. Zero
limits mean unlimited. See the [Go README](https://github.com/brmmm3/scandir-rs/blob/master/goscandir/README.md)
for tests, examples, and the current cgo setup.
