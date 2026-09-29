# jscandir-ffm

Java FFM (Foreign Function & Memory API) wrapper for the `scandir-rs` directory scanner.

Requires Java 22+ (FFM API is in preview in Java 19-21, standardized in 22).

## Usage

```java
import com.scandir.JScandir;
import com.scandir.options.Options;
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
mvn compile
```

## Testing

```bash
mvn test
```

## Requirements

- Java 22+ (for FFM API)
- `libcscandir.so` (or `cscandir.dll` on Windows) available on the library path
  - Set `cscandir.lib.path` system property to specify custom location
  - Or place library in standard locations: `../../target/release`, `../target/release`, `target/release`, `target/debug`

## API

- `JScandir.collect(root, options)` - Collect entries with metadata
- `JScandir.count(root, options)` - Aggregate statistics only
- `JScandir.walk(root, options)` - Entries grouped by directory

## Options

```java
Options opts = Options.defaultOptions()
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

Uses Java 22's Foreign Function & Memory API (JEP 454) for zero-copy interop with the C library. The `ScandirLinker` class provides the FFM bindings to the C API.

## License

MIT OR Apache-2.0
## Examples

```bash
# Build the project
mvn compile

# Run examples (requires libcscandir.so)
export CSCANDIR_LIB_PATH=/path/to/libcscandir.so
mvn exec:java -Dexec.mainClass="com.scandir.examples.ScandirCollect" -Dexec.args="."
mvn exec:java -Dexec.mainClass="com.scandir.examples.ScandirWalk" -Dexec.args="."
mvn exec:java -Dexec.mainClass="com.scandir.examples.ScandirCount" -Dexec.args="."
mvn exec:java -Dexec.mainClass="com.scandir.examples.ScandirFilter" -Dexec.args="."
```

Available examples:
- `ScandirCollect` - Basic entry collection with extended metadata
- `ScandirWalk` - Directory tree walk with per-directory breakdown
- `ScandirCount` - Fast statistics only (no entry materialization)
- `ScandirFilter` - File/directory filtering with patterns

