---
layout: default
title: Home
---

`scandir-rs` is a parallel directory-tree scanner with APIs for collecting file
metadata, walking directory trees, and counting entries. These guides show how
to use it from supported languages in your own projects.

Version 3.0.0 is released for supported platforms. See
[getting started](getting-started.html) and the language guides for package
installation and platform-specific native dependencies.

## Choose a language

| Language | Binding | Guide |
| --- | --- | --- |
| Rust | Native crate | [Rust](rust.html) |
| Python | PyO3 extension module | [Python](python.html) |
| C | C ABI | [C](c.html) |
| C++ | Rust API through the `cxx` bridge | [C++](cpp.html) |
| Go | cgo wrapper over the C ABI | [Go](go.html) |
| Java | JNI wrapper over the C ABI; Java 25+ | [Java JNI](java-jni.html) |
| Java | Foreign Function & Memory (FFM) wrapper; Java 25+ | [Java FFM](java-ffm.html) |
| C# | .NET P/Invoke wrapper over the C ABI | [C#](csharp.html) |

Start with [getting started](getting-started.html) for installation and binding
selection, or see [shared API behavior](api.html) for defaults, paths, metadata,
and errors.

## Project links

- [Source repository](https://github.com/brmmm3/scandir-rs)
- [Changelog](https://github.com/brmmm3/scandir-rs/blob/master/CHANGELOG.md)
- [MIT license](https://github.com/brmmm3/scandir-rs/blob/master/LICENSE-MIT) or
  [Apache-2.0 license](https://github.com/brmmm3/scandir-rs/blob/master/LICENSE-APACHE)
