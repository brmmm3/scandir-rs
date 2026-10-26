# goscandir

Go bindings for the `scandir-rs` C ABI. This module requires cgo and a built
`libcscandir` library.

## Build the native library

From the repository root:

```sh
cargo build -p cscandir --release
```

The Go package expects the library in `../target/release` relative to this
module. Override `CGO_CFLAGS` and `CGO_LDFLAGS` to use a custom header/library
location.

## Test and build

Run from this directory:

```sh
go test ./...
go test -race ./...
go vet ./...
go build ./examples/...
```

The four examples are separate commands and accept an optional root path:

```sh
go run ./examples/collect [ROOT]
go run ./examples/count [ROOT]
go run ./examples/walk [ROOT]
go run ./examples/filter [ROOT]
```

`Collect` copies native entries into Go values and includes a synthetic root
entry. Its duration is measured in Go; use `Count` for native aggregate values
such as file counts, hardlinks, and disk usage. Set `Options.ReturnType` to
`ReturnTypeExt` to request the metadata needed for accurate extended counting.
The C API has no walk endpoint, so `Walk` groups a materialized `Collect`
result in Go and is not streaming.
