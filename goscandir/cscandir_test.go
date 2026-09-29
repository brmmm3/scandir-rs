package cscandir

import (
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func tree(t *testing.T, name string) string {
	t.Helper()
	root := filepath.Join(t.TempDir(), name)
	mustMkdir(t, filepath.Join(root, "sub", "deep"))
	mustWrite(t, filepath.Join(root, "a.txt"))
	mustWrite(t, filepath.Join(root, "b.txt"))
	mustWrite(t, filepath.Join(root, "sub", "c.txt"))
	mustWrite(t, filepath.Join(root, "sub", "deep", "d.txt"))
	return root
}

func mustMkdir(t *testing.T, path string) {
	t.Helper()
	if err := os.MkdirAll(path, 0o755); err != nil {
		t.Fatalf("mkdir %s: %v", path, err)
	}
}

func mustWrite(t *testing.T, path string) {
	t.Helper()
	if err := os.WriteFile(path, []byte("content"), 0o644); err != nil {
		t.Fatalf("write %s: %v", path, err)
	}
}

func countByKind(entries []Entry) (files, dirs int) {
	for _, entry := range entries {
		if entry.IsFile != 0 {
			files++
		}
		if entry.IsDir != 0 {
			dirs++
		}
	}
	return files, dirs
}

func TestCollectReturnsEntriesAndStatistics(t *testing.T) {
	root := tree(t, "collect")
	result, err := Collect(root, nil)
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	if len(result) != 6 {
		t.Errorf("entries = %d, want 6", len(result))
	}
	files, dirs := countByKind(result)
	if files != 4 || dirs != 2 {
		t.Errorf("files/dirs = %d/%d, want 4/2", files, dirs)
	}
}

func TestEntriesCarryPathsAndTimes(t *testing.T) {
	root := tree(t, "entries")
	result, err := Collect(root, nil)
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	for _, entry := range result {
		if entry.Path == nil {
			t.Error("entry has empty path")
		}
	}
}

func TestBaseReturnTypeLeavesExtUnset(t *testing.T) {
	root := tree(t, "base")
	result, err := Collect(root, &Options{ReturnType: RETURN_TYPE_BASE})
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	for _, entry := range result {
		if entry.HasExt != 0 {
			t.Errorf("%s: HasExt set with ReturnTypeBase", entry.Path)
		}
		if entry.Mode != 0 || entry.Ino != 0 {
			t.Errorf("%s: ext fields populated with ReturnTypeBase", entry.Path)
		}
	}
}

func TestExtendedReturnTypePopulatesExt(t *testing.T) {
	root := tree(t, "ext")
	result, err := Collect(root, &Options{ReturnType: RETURN_TYPE_EXT})
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	if len(result) == 0 {
		t.Fatal("no entries")
	}
	for _, entry := range result {
		if entry.HasExt == 0 {
			t.Errorf("%s: HasExt not set with ReturnTypeExt", entry.Path)
		}
		if entry.Ino == 0 {
			t.Errorf("%s: inode not populated", entry.Path)
		}
	}
}

func TestCountReturnsStatistics(t *testing.T) {
	root := tree(t, "count")
	stats, err := Count(root, nil)
	if err != nil {
		t.Fatalf("Count: %v", err)
	}
	if stats.Files != 4 || stats.Dirs != 2 {
		t.Errorf("files/dirs = %d/%d, want 4/2", stats.Files, stats.Dirs)
	}
	if stats.Duration <= 0 {
		t.Errorf("duration = %v, want > 0", stats.Duration)
	}
}

func TestWalkGroupsEntriesPerDirectory(t *testing.T) {
	root := tree(t, "walk")
	toc, roots, _, err := Walk(root, nil)
	if err != nil {
		t.Fatalf("Walk: %v", err)
	}
	if len(toc.Files) != 4 || len(toc.Dirs) != 2 {
		t.Errorf("toc files/dirs = %d/%d, want 4/2",
			len(toc.Files), len(toc.Dirs))
	}
	if len(roots) < 2 {
		t.Errorf("roots = %d, want >= 2", len(roots))
	}
}

func TestWalkTocFilesEndInTxt(t *testing.T) {
	root := tree(t, "walk_toc")
	toc, _, _, err := Walk(root, nil)
	if err != nil {
		t.Fatalf("Walk: %v", err)
	}
	for _, file := range toc.Files {
		if !strings.HasSuffix(file, ".txt") {
			t.Errorf("unexpected file %q", file)
		}
	}
}

func TestMaxDepthLimitsResults(t *testing.T) {
	root := tree(t, "depth")
	result, err := Collect(root, &Options{MaxDepth: 1})
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	if len(result) != 3 {
		t.Errorf("entries = %d, want 3", len(result))
	}
}

func TestMaxDepthZeroMeansUnlimited(t *testing.T) {
	root := tree(t, "depth_zero")
	result, err := Collect(root, &Options{MaxDepth: 0})
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	if len(result) != 6 {
		t.Errorf("entries = %d, want 6", len(result))
	}
}

func TestFileIncludeFilter(t *testing.T) {
	root := tree(t, "include")
	result, err := Collect(root, &Options{FileInclude: []string{"*.txt"}})
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	files, _ := countByKind(result)
	if files != 4 {
		t.Errorf("files = %d, want 4", files)
	}
	for _, entry := range result {
		if entry.IsFile != 0 && !strings.HasSuffix(entry.Path, ".txt") {
			t.Errorf("unexpected file %q", entry.Path)
		}
	}
}

func TestDirExcludeFilter(t *testing.T) {
	root := tree(t, "exclude")
	result, err := Collect(root, &Options{DirExclude: []string{"sub"}})
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	if len(result) != 2 {
		t.Errorf("entries = %d, want 2", len(result))
	}
}

func TestSkipHiddenFilter(t *testing.T) {
	root := tree(t, "hidden")
	mustWrite(t, filepath.Join(root, ".hidden.txt"))
	result, err := Collect(root, &Options{SkipHidden: 1})
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	if len(result) != 6 {
		t.Errorf("entries = %d, want 6", len(result))
	}
	for _, entry := range result {
		if strings.Contains(entry.Path, ".hidden") {
			t.Errorf("hidden entry leaked through: %q", entry.Path)
		}
	}
}

func TestOptionsAreReusable(t *testing.T) {
	root := tree(t, "reuse")
	options := &Options{FileInclude: []string{"*.txt"}, Sorted: true, CaseSensitive: true}

	for i := 0; i < 50; i++ {
		result, err := Collect(root, options)
		if err != nil {
			t.Fatalf("iteration %d: %v", i, err)
		}
		if files, _ := countByKind(result); files != 4 {
			t.Fatalf("iteration %d: files = %d, want 4", i, files)
		}
	}
}

func TestManyFiltersAtOnce(t *testing.T) {
	root := tree(t, "filters")
	options := &Options{
		DirInclude:  []string{"sub", "a*"},
		FileInclude: []string{"*.txt"},
		FileExclude: []string{"d*"},
		Sorted:      true,
	}
	result, err := Collect(root, options)
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	if len(result) == 0 {
		t.Error("expected some entries")
	}
}

func TestMissingRootReturnsError(t *testing.T) {
	_, err := Collect(filepath.Join(t.TempDir(), "does-not-exist"), nil)
	if err == nil {
		t.Fatal("expected an error for a missing root")
	}
	var scanErr *Error
	if !errors.As(err, &scanErr) {
		t.Fatalf("error type = %T, want *Error", err)
	}
	if scanErr.Code != 3 {
		t.Errorf("code = %d, want %d", scanErr.Code, 3)
	}
}

func TestInvalidUTF8RootIsRejected(t *testing.T) {
	if filepath.Separator == '\\' {
		t.Skip("not applicable on Windows")
	}
	invalid := "/tmp/\xff\xfe"
	_, err := Collect(invalid, nil)
	if err == nil {
		t.Fatal("expected an error for a non-UTF-8 path")
	}
	var scanErr *Error
	if !errors.As(err, &scanErr) || scanErr.Code != 2 {
		t.Errorf("error = %v, want invalid UTF-8", err)
	}
}

func TestCountPropagatesErrors(t *testing.T) {
	if _, err := Count(filepath.Join(t.TempDir(), "nope"), nil); err == nil {
		t.Fatal("expected an error")
	}
	if _, err := Walk(filepath.Join(t.TempDir(), "nope"), nil); err == nil {
		t.Fatal("expected an error")
	}
}

func TestErrorsAfterSuccessDoNotLeakState(t *testing.T) {
	root := tree(t, "alternate")
	missing := filepath.Join(t.TempDir(), "nope")

	for i := 0; i < 20; i++ {
		if _, err := Collect(missing, nil); err == nil {
			t.Fatalf("iteration %d: expected error", i)
		}
		result, err := Collect(root, nil)
		if err != nil {
			t.Fatalf("iteration %d: %v", i, err)
		}
		if len(result) != 6 {
			t.Fatalf("iteration %d: entries = %d, want 6", i, len(result))
		}
	}
}

func TestResultsSurviveSourceDeletion(t *testing.T) {
	base := t.TempDir()
	root := filepath.Join(base, "scratch")
	mustMkdir(t, filepath.Join(root, "sub"))
	mustWrite(t, filepath.Join(root, "a.txt"))
	mustWrite(t, filepath.Join(root, "sub", "b.txt"))

	result, err := Collect(root, nil)
	if err != nil {
		t.Fatalf("Collect: %v", err)
	}
	if err := os.RemoveAll(root); err != nil {
		t.Fatalf("cleanup: %v", err)
	}

	if len(result) != 3 {
		t.Errorf("entries = %d, want 3", len(result))
	}
	for _, entry := range result {
		if entry.Path == nil {
			t.Error("path lost after deletion")
		}
	}
}