# jscandir-jni

Java JNI wrapper for the `scandir-rs` directory scanner.

Uses traditional Java Native Interface (JNI) for C interop. Compatible with Java 8+.

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
# Build native library first
cd ../../cscandir && cargo build --release

# Build Java project (includes native JNI library compilation)
mvn compile
```

## Testing

```bash
mvn test
```

## Requirements

- Java 17+
- `libcscandir.so` (or `cscandir.dll` on Windows) and `libjscandir.so` (or `jscandir.dll`)
  - Set `jscandir.lib.path` system property to specify custom location
  - Or place libraries in standard locations: `../../target/release`, `../target/release`,
    `target/release`, `target/debug`

## API

- `JScandir.collect(root, options)` - Collect entries with metadata
- `JScandir.count(root, options)` - Aggregate statistics only
- `JScandir.walk(root, options)` - Entries grouped by directory

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
The `src/main/cpp/jscandir_jni.cpp` file contains the JNI bridge code that calls the C `cscandir` library.
The native library `libjscandir.so` (built by native-maven-plugin) wraps the C `libcscandir.so`.

## License

MIT OR Apache-2.0
