# jscandir-jni

Java JNI wrapper for the `scandir-rs` directory scanner.

Uses the Java Native Interface (JNI) for C interop. Requires Java 17+.

## Usage

```java
import com.scandir.jni.JScandir;
import com.scandir.options.Options;
import com.scandir.options.ReturnType;
import com.scandir.result.CollectResult;
import java.nio.file.Path;

Path root = Path.of("/usr");
Options options = Options.defaultOptions()
    .sorted(true)
    .skipHidden(true)
    .returnType(ReturnType.EXT);

CollectResult result = JScandir.collect(Path.of("/usr"), options);

for (Entry entry : result.getEntries()) {
    System.out.println(entry.getPath() + " (" + (entry.isDir() ? "DIR" : "FILE") + ")");
}
```

## Building

```bash
# From the repository root, build the C ABI library first
cargo build -p cscandir --release

# From this module directory, Maven compiles the JNI bridge and Java sources
cd jscandir-jni
mvn compile
```

## Testing

```bash
mvn test
```

## Requirements

- Java 17+
- `libcscandir.so` (or `cscandir.dll` on Windows) and `libjscandir.so` (or `jscandir.dll`)
  - Set `jscandir.lib.path` to the JNI bridge file or its containing directory
  - Maven creates the bridge at `target/native/libjscandir.so` on Linux
  - The bridge links to the workspace library at `../target/release/libcscandir.so`

## API

- `JScandir.collect(root, options)` - Collect entries with metadata
- `JScandir.count(root, options)` - Aggregate statistics only
- `JScandir.walk(root, options)` - Groups one full collect result by parent directory in Java; the C ABI has no native walk endpoint

## Options

```java
Options opts = new Options()
    .sorted(true)
    .skipHidden(true)
    .maxDepth(10)
    .maxFileCnt(1000)
    .dirInclude("src", "include")
    .dirExclude("target", "build")
    .fileInclude("*.java", "*.kt")
    .fileExclude("*.class")
    .caseSensitive(true)
    .followLinks(false)
    .returnType(ReturnType.EXT);
```

## Architecture

Uses traditional JNI for C interop.
The `src/main/cpp/jscandir_jni.cpp` bridge calls the C `cscandir_collect` and `cscandir_count` APIs. JNI incurs a per-entry Java object and string conversion cost; when processing large trees, `count` is cheaper because it does not allocate entry objects. JNI UTF-16 strings are converted to/from the C API's UTF-8 explicitly. C-owned buffers are freed after conversion.

The Maven build creates `target/native` and invokes `g++` with the active JDK's JNI headers, then links the bridge against the release C library in `../target/release`. Build that library first from the repository root with `cargo build -p cscandir --release`. The bridge build currently targets Linux; Windows and macOS require platform-specific compiler, header, linker, and library-name settings.

## Examples

```bash
mvn compile
for example in ScandirCollect ScandirCount ScandirWalk ScandirFilter; do
  java --enable-native-access=ALL-UNNAMED \
    -Djscandir.lib.path=target/native -cp target/classes \
    "com.scandir.examples.$example" .
done
# To run against a mounted CIFS tree:
java --enable-native-access=ALL-UNNAMED \
  -Djscandir.lib.path=target/native -cp target/classes \
  com.scandir.examples.CifsCtimeCheck /mnt/smb
```

The CIFS check can only be validated against an actual SMB mount. The current C entry ABI exposes ctime but not Rust's birth-time field.

## License

MIT OR Apache-2.0
