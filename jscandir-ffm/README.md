# jscandir-ffm

Java FFM (Foreign Function & Memory API) wrapper for the `scandir-rs` directory scanner.

Requires Java 25 LTS+. The FFM API was standardized in Java 22; this module targets Java 25.

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

- Java 22+ (this module is currently compiled with Java 25 for the stable FFM API)
- `libcscandir.so` (or the platform library) available. Set `cscandir.lib.path` to either its full file path or containing directory.
- Build the native library from the repository root with `cargo build -p cscandir --release`.

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

## Architecture and performance

Uses the stable Java Foreign Function & Memory API to call the supported `cscandir_collect` and `cscandir_count` functions.
Output pointers are decoded in place into Java result objects and C-owned output allocations are freed in `finally` blocks.
FFM avoids the extra JNI bridge but collecting still materializes Java entry objects and copies UTF-8 names into Java strings.
The C ABI is synchronous and does not export `walk`; `JScandir.walk` groups the single collect result in Java (it is not a streaming walk).

## License

MIT OR Apache-2.0

## Examples

```bash
# Build the project
mvn compile

# Run examples (requires libcscandir.so)
LIB=/path/to/libcscandir.so
for example in ScandirCollect ScandirWalk ScandirCount ScandirFilter; do
  java --enable-native-access=ALL-UNNAMED -Dcscandir.lib.path="$LIB" \
    -cp target/classes "com.scandir.examples.$example" .
done
# To run against a mounted CIFS tree:
java --enable-native-access=ALL-UNNAMED -Dcscandir.lib.path="$LIB" \
  -cp target/classes com.scandir.examples.CifsCtimeCheck /mnt/smb
```

Available examples:

- `ScandirCollect` - Basic entry collection with extended metadata
- `ScandirWalk` - Directory tree walk with per-directory breakdown
- `ScandirCount` - Fast statistics only (no entry materialization)
- `ScandirFilter` - File/directory filtering with patterns
- `CifsCtimeCheck` - ctime check with a `unix:ctime` cross-check (requires a mounted CIFS tree to validate CIFS behavior)
