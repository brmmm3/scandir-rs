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

- Call `cscandir_options_init()` before customizing `cscandir_options`.
- All strings must be valid UTF-8.
- Free output buffers with:
  - `cscandir_free_entry_list()`
  - `cscandir_free_string_list()`
  - `cscandir_free_error()`
