---
title: Java (FFM)
---

The `jscandir-ffm` module calls the C ABI using Java's Foreign Function &
Memory API, without a JNI bridge. The module is compiled for Java 25 and
requires a Java 25+ toolchain, Maven, and a built `cscandir` library.

## Build and test

From the repository root:

```sh
cargo build -p cscandir --release
cd jscandir-ffm
mvn test
```

The FFM wrapper loads `libcscandir` directly. Configure
`cscandir.lib.path` with the library file or its containing directory when it
is not in the module's expected `../target/release` location. FFM calls require
native access to be enabled when launching Java:

```sh
java --enable-native-access=ALL-UNNAMED \
  -Dcscandir.lib.path=/path/to/libcscandir.so \
  -cp target/classes com.scandir.examples.ScandirCollect /var/log
```

## Use the API

```java
import com.scandir.JScandir;
import com.scandir.options.Options;
import com.scandir.options.ReturnType;
import com.scandir.result.CollectResult;
import com.scandir.result.Entry;
import java.nio.file.Path;

Options options = Options.defaultOptions()
    .sorted(true)
    .skipHidden(true)
    .returnType(ReturnType.EXT);

CollectResult result = JScandir.collect(Path.of("/var/log"), options);
for (Entry entry : result.getEntries()) {
    System.out.println(entry.getPath());
}
result.getErrors().forEach(System.err::println);
```

`JScandir.count(root, options)` returns aggregate statistics without
materializing entries. The provided `walk()` groups a complete collect result
in Java; it is not a native or streaming walk. Fatal failures throw
`JScandirError`; per-entry traversal errors are available on the result.

The wrapper decodes native buffers into Java values and releases C-owned output
memory after conversion. See the [FFM README](https://github.com/brmmm3/scandir-rs/blob/master/jscandir-ffm/README.md)
for supported options and runnable examples.
