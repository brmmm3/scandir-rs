---
title: Getting started
---

## Pick an API

| Need | Suggested interface | Notes |
| --- | --- | --- |
| Native Rust traversal and background-worker controls | Rust | Direct access to `Count`, `Walk`, and `Scandir`. |
| Python iteration and Python `DirEntry`-style objects | Python | Native extension; releases the GIL while scanning. |
| A C interface for a native application | C | Synchronous `collect` and `count` functions. |
| C++ calling Rust without a C ABI wrapper | C++ | Uses the `cxx` bridge and the Rust scanner directly. |
| Go | Go | cgo wrapper around the synchronous C ABI; requires a native library. |
| Java 25 projects using established native interop | Java JNI | Builds a JNI bridge and the C ABI library. |
| Java 25 projects using the stable FFM API | Java FFM | Calls the C ABI directly, without a JNI bridge. |
| .NET applications | C# | Managed wrapper using P/Invoke and a native C ABI library. |

The C ABI currently exposes only synchronous `collect` and `count`. Go, JNI,
FFM, and C# walk-style helpers group a fully collected result in the wrapper;
they are not streaming. Rust provides the native walk and background-worker
APIs; Python also exposes background workers and iteration. The current C++
binding provides synchronous scan and count operations.

## Installation and native prerequisites

Install the 3.0.0 Rust and Python packages through Cargo and PyPI; see their
language guides for the exact commands. The C, Go, Java, and C# bindings use the
`cscandir` native library. Install the 3.0.0 package for your target platform,
or build it from a source checkout with:

```sh
cargo build -p cscandir --release
```

This produces `target/release/libcscandir.so` on Linux,
`target/release/libcscandir.dylib` on macOS, or
`target/release/cscandir.dll` on Windows. Place the library where the operating
system loader can find it, or configure the binding-specific search path
described in its guide. The JNI bridge build documented in this repository
currently targets Linux.

All packages are released at version 3.0.0 for their supported platforms. For
wrappers that link to `cscandir`, use the matching native package for the target
operating system and architecture, and configure its library path as described
in the binding guide. The source-build instructions remain available for local
development and custom builds.

## GitHub Pages setup

The site is a Jekyll project in the repository's `docs/` directory and uses the
Cayman theme. To publish it, open the repository's
**Settings → Pages**, select the desired branch and the `/docs` folder as the
build source, and save. GitHub Pages will build the site; no local Jekyll setup
is required for ordinary content edits.
