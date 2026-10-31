---
title: Java (JNI)
---

The `jscandir-jni` module provides a Java API using JNI to call the C ABI. It
requires Java 25+, Maven, a C++ compiler, and the `cscandir` native library.
The JNI bridge build in this repository currently targets Linux.

## Build and test

From the repository root, build the native C ABI library first, then compile
and test the Java module:

```sh
cargo build -p cscandir --release
cd jscandir-jni
mvn test
```

Maven compiles the JNI bridge into `target/native/libjscandir.so` and links it
against `../target/release/libcscandir.so`. The C ABI library must remain
discoverable at runtime as well. Use platform-specific compiler, linker, and
library names when adapting the bridge build beyond the currently documented
Linux configuration.

## Use the API

```java
import com.scandir.jni.JScandir;
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

Use `JScandir.count(root, options)` to get aggregate statistics without
creating one Java entry object per path. `JScandir.walk()` groups a full collect
result in Java; the C ABI has no native walk operation, so it is not streaming.
Fatal native failures throw `JScandirException`; per-entry traversal issues
are returned separately.

## Run an application

For the in-repository examples, run from `jscandir-jni/` after `mvn compile`:

```sh
java -Djscandir.lib.path=target/native -cp target/classes \
  com.scandir.examples.ScandirCollect /var/log
```

`jscandir.lib.path` may name the JNI library or its containing directory. The
JNI library also depends on `libcscandir` at runtime. See the
[JNI README](https://github.com/brmmm3/scandir-rs/blob/master/jscandir-jni/README.md)
for option setters, remaining examples, and the native build configuration.
