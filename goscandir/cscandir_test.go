package cscandir

import (
	"errors"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

func tree(t *testing.T) string {
	t.Helper()
	root := filepath.Join(t.TempDir(), "fixture")
	for _, dir := range []string{"sub/deep"} {
		if err := os.MkdirAll(filepath.Join(root, dir), 0o755); err != nil {
			t.Fatal(err)
		}
	}
	for _, file := range []string{"a.txt", "b.txt", "sub/c.txt", "sub/deep/d.txt"} {
		if err := os.WriteFile(filepath.Join(root, file), []byte("content"), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	return root
}

func TestCollectAndCount(t *testing.T) {
	root := tree(t)
	result, err := Collect(root, nil)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.Entries) != 7 { // 4 files, 2 descendant directories, synthetic root.
		t.Fatalf("Collect returned %d entries, want 7", len(result.Entries))
	}
	if result.Entries[0].Path != filepath.Base(root) {
		t.Errorf("synthetic root path = %q, want %q", result.Entries[0].Path, filepath.Base(root))
	}
	if !result.Entries[0].IsDir {
		t.Errorf("first entry is not the synthetic root directory: %+v", result.Entries[0])
	}
	if len(result.Errors) != 0 {
		t.Errorf("Collect errors = %v", result.Errors)
	}
	if result.Statistics.Files != 0 || result.Statistics.Dirs != 0 || result.Statistics.Size != 0 {
		t.Errorf("Collect populated native-only aggregate statistics: %+v", result.Statistics)
	}
	if result.Statistics.Duration <= 0 {
		t.Errorf("Collect duration = %v, want positive", result.Statistics.Duration)
	}

	stats, err := Count(root, nil)
	if err != nil {
		t.Fatal(err)
	}
	if stats.Files != 4 || stats.Dirs != 2 {
		t.Errorf("Count files/dirs = %d/%d, want 4/2", stats.Files, stats.Dirs)
	}
	if stats.Duration <= 0 {
		t.Errorf("Count duration = %v, want positive", stats.Duration)
	}
}

func TestCountExtendedStatisticsMatchNativeAPI(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("hardlink metadata assertion is POSIX-specific")
	}
	root := tree(t)
	if err := os.Link(filepath.Join(root, "a.txt"), filepath.Join(root, "sub", "a-hardlink.txt")); err != nil {
		t.Fatal(err)
	}

	options := &Options{ReturnType: ReturnTypeExt}
	collected, err := Collect(root, options)
	if err != nil {
		t.Fatal(err)
	}
	stats, err := Count(root, options)
	if err != nil {
		t.Fatal(err)
	}
	if stats.Files != 4 || stats.Hlinks != 1 || stats.Dirs != 2 {
		t.Errorf("extended Count files/hardlinks/dirs = %d/%d/%d, want 4/1/2", stats.Files, stats.Hlinks, stats.Dirs)
	}
	// Hardlink accounting affects the file count, not size/usage totals.
	wantSize := uint64(5*len("content") + 2*4096)
	if stats.Size != wantSize {
		t.Errorf("extended Count size = %d, want %d", stats.Size, wantSize)
	}
	var wantUsage uint64
	for i, entry := range collected.Entries {
		if i == 0 { // Ignore Collect's synthetic root entry, as Count does.
			continue
		}
		if entry.IsFile {
			wantUsage += entry.Blocks << 9
		} else {
			wantUsage += 4096
		}
	}
	if stats.Usage != wantUsage {
		t.Errorf("extended Count usage = %d, want %d from collected metadata", stats.Usage, wantUsage)
	}
}

func TestFiltersAndMetadata(t *testing.T) {
	root := tree(t)
	result, err := Collect(root, &Options{
		FileInclude: []string{"*.txt"},
		DirExclude:  []string{"sub/deep"},
		ReturnType:  ReturnTypeExt,
		Sorted:      true,
	})
	if err != nil {
		t.Fatal(err)
	}
	for _, entry := range result.Entries {
		if entry.Path == "sub/deep" || entry.Path == "sub/deep/d.txt" {
			t.Errorf("excluded path returned: %q", entry.Path)
		}
		if entry.IsFile && !strings.HasSuffix(entry.Path, ".txt") {
			t.Errorf("file include filter returned %q", entry.Path)
		}
		if entry.Path != filepath.Base(root) && !entry.HasExt {
			t.Errorf("extended metadata missing for %q", entry.Path)
		}
	}
}

func TestWalkGroupsEntries(t *testing.T) {
	root := tree(t)
	result, err := Walk(root, nil)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.Toc.Files) != 4 || len(result.Toc.Dirs) != 2 {
		t.Errorf("merged TOC files/dirs = %d/%d, want 4/2", len(result.Toc.Files), len(result.Toc.Dirs))
	}
	if len(result.Roots) != 3 {
		t.Fatalf("Walk returned %d directory groups, want 3", len(result.Roots))
	}
	want := map[string]bool{"": true, "sub": true, "sub/deep": true}
	for _, dir := range result.Roots {
		if !want[dir.Dir] {
			t.Errorf("unexpected directory group %q", dir.Dir)
		}
		delete(want, dir.Dir)
	}
	if len(want) != 0 {
		t.Errorf("missing directory groups: %v", want)
	}
	for _, dir := range result.Roots {
		for _, name := range dir.Toc.Files {
			if strings.Contains(name, "/") {
				t.Errorf("Walk local file name should be basename, got %q", name)
			}
		}
	}
}

func TestUnicodeFilenames(t *testing.T) {
	root := filepath.Join(t.TempDir(), "unicode-目录")
	if err := os.MkdirAll(filepath.Join(root, "子目录"), 0o755); err != nil {
		t.Fatal(err)
	}
	for _, file := range []string{"雪だるま-☃.txt", "子目录/файл.txt"} {
		if err := os.WriteFile(filepath.Join(root, file), []byte("unicode"), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	result, err := Collect(root, &Options{Sorted: true})
	if err != nil {
		t.Fatal(err)
	}
	want := map[string]bool{"unicode-目录": true, "子目录": true, "雪だるま-☃.txt": true, "子目录/файл.txt": true}
	for _, entry := range result.Entries {
		delete(want, entry.Path)
	}
	if len(want) != 0 {
		t.Errorf("missing Unicode paths: %v", want)
	}
}

func TestInvalidInputsAndErrorsIs(t *testing.T) {
	if _, err := Collect("/tmp/nul\x00path", nil); !errors.Is(err, ErrNulByte) {
		t.Errorf("NUL path error = %v, want ErrNulByte", err)
	}
	if _, err := Collect(string([]byte{'/', 0xff}), nil); !errors.Is(err, ErrInvalidUTF8) {
		t.Errorf("invalid UTF-8 path error = %v, want ErrInvalidUTF8", err)
	}
	if _, err := Collect(t.TempDir(), &Options{FileInclude: []string{"bad\x00glob"}}); !errors.Is(err, ErrNulByte) {
		t.Errorf("NUL glob error = %v, want ErrNulByte", err)
	}
	if _, err := Collect(t.TempDir(), &Options{ReturnType: 99}); !errors.Is(err, ErrInvalidArgument) {
		t.Errorf("invalid return type error = %v, want ErrInvalidArgument", err)
	}
	missing := filepath.Join(t.TempDir(), "not-found")
	if _, err := Collect(missing, nil); !errors.Is(err, ErrScan) {
		t.Errorf("missing root error = %v, want ErrScan", err)
	}
}

func TestResultsRemainIndependentOfNativeBuffers(t *testing.T) {
	root := tree(t)
	result, err := Collect(root, nil)
	if err != nil {
		t.Fatal(err)
	}
	firstPath := result.Entries[0].Path
	if err := os.RemoveAll(root); err != nil {
		t.Fatal(err)
	}
	if result.Entries[0].Path != firstPath {
		t.Errorf("copied result changed after freeing native data")
	}
}

func BenchmarkCollect(b *testing.B) {
	root := b.TempDir()
	for i := 0; i < 100; i++ {
		if err := os.WriteFile(filepath.Join(root, "file-"+strings.Repeat("x", i%10)+string(rune('a'+i%26))), []byte("data"), 0o644); err != nil {
			b.Fatal(err)
		}
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if _, err := Collect(root, nil); err != nil {
			b.Fatal(err)
		}
	}
}
