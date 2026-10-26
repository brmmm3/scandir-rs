// Package cscandir provides Go bindings for the scandir-rs directory scanner.
//
// It wraps the C ABI exported by the cscandir crate. Returned memory is copied
// into Go values, so results remain valid after the native buffers are freed.
// The package requires cgo and links against libcscandir. By default it looks
// for the library in ../target/release relative to this package; override the
// compiler and linker flags with CGO_CFLAGS and CGO_LDFLAGS when needed.
package cscandir

/*
#cgo CFLAGS: -I${SRCDIR}/../cscandir/include
#cgo LDFLAGS: -L${SRCDIR}/../target/release -Wl,-rpath,${SRCDIR}/../target/release -lcscandir

#include <stdlib.h>
#include "cscandir.h"
*/
import "C"

import (
	"errors"
	"fmt"
	"path"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"time"
	"unicode/utf8"
	"unsafe"
)

// Error codes reported by the C library.
const (
	ErrCodeOK              int32 = C.CSCANDIR_OK
	ErrCodeInvalidArgument int32 = C.CSCANDIR_ERR_INVALID_ARGUMENT
	ErrCodeInvalidUTF8     int32 = C.CSCANDIR_ERR_INVALID_UTF8
	ErrCodeScan            int32 = C.CSCANDIR_ERR_SCAN
	ErrCodeNulByte         int32 = C.CSCANDIR_ERR_NUL_BYTE
	ErrCodePanic           int32 = C.CSCANDIR_ERR_PANIC
)

// ReturnType selects how much detail each entry carries.
type ReturnType uint32

const (
	// ReturnTypeBase reports path, type flags, timestamps and size.
	ReturnTypeBase ReturnType = C.CSCANDIR_RETURN_BASE
	// ReturnTypeExt additionally populates the extended stat fields.
	ReturnTypeExt ReturnType = C.CSCANDIR_RETURN_EXT
)

// Error is returned when a scan fails.
type Error struct {
	Code int32
	Msg  string
}

func (e *Error) Error() string {
	return fmt.Sprintf("cscandir: %s (code %d)", e.Msg, e.Code)
}

// Is allows errors.Is to compare errors by their native error code.
func (e *Error) Is(target error) bool {
	other, ok := target.(*Error)
	return ok && other.Code == e.Code
}

// Sentinel errors match any error carrying the corresponding native code.
var (
	ErrInvalidArgument error = &Error{Code: ErrCodeInvalidArgument, Msg: "invalid argument"}
	ErrInvalidUTF8     error = &Error{Code: ErrCodeInvalidUTF8, Msg: "invalid UTF-8"}
	ErrScan            error = &Error{Code: ErrCodeScan, Msg: "scan failed"}
	ErrNulByte         error = &Error{Code: ErrCodeNulByte, Msg: "NUL byte in string"}
	ErrPanic           error = &Error{Code: ErrCodePanic, Msg: "native panic"}
)

// Options configures a scan. The zero value matches the C API defaults.
// MaxDepth and MaxFileCnt values of zero mean unlimited.
type Options struct {
	Sorted      bool
	SkipHidden  bool
	MaxDepth    uint64
	MaxFileCnt  uint64
	DirInclude  []string
	DirExclude  []string
	FileInclude []string
	FileExclude []string

	CaseSensitive bool
	FollowLinks   bool
	ReturnType    ReturnType
}

// Statistics summarizes a scan. Count returns native aggregate counters and
// per-entry errors; Collect only populates Duration and exposes traversal
// errors separately on CollectResult.Errors. Fatal failures are Go errors.
type Statistics struct {
	Dirs     int32
	Files    int32
	Slinks   int32
	Hlinks   int32
	Devices  int32
	Pipes    int32
	Size     uint64
	Usage    uint64
	Duration float64
	Errors   []string
}

// Entry is one directory entry. Descendant paths are relative to the scan
// root. Collect includes a synthetic root entry using the root's basename.
type Entry struct {
	Path      string
	IsSymlink bool
	IsDir     bool
	IsFile    bool

	// Ctime, Mtime and Atime are seconds since the Unix epoch.
	Ctime float64
	Mtime float64
	Atime float64

	Size uint64

	// HasExt reports whether the extended fields below were populated.
	HasExt  bool
	Mode    uint32
	Ino     uint64
	Dev     uint64
	Nlink   uint64
	Blksize uint64
	Blocks  uint64
	UID     uint32
	GID     uint32
	Rdev    uint64
}

// Toc groups relative entry paths by type. Paths include the directory path
// within the scan root; Errors contains traversal errors from the scan.
type Toc struct {
	Dirs     []string
	Files    []string
	Symlinks []string
	Other    []string
	Errors   []string
}

// WalkEntry describes entries directly inside one directory. Dir is empty for
// the scan root.
type WalkEntry struct {
	Dir string
	Toc Toc
}

// CollectResult is returned by Collect. Statistics.Duration is measured by
// the wrapper; use Count for native aggregate counters and disk usage.
type CollectResult struct {
	Entries    []Entry
	Errors     []string
	Statistics Statistics
}

// WalkResult is returned by Walk.
type WalkResult struct {
	Toc        Toc
	Roots      []WalkEntry
	Statistics Statistics
}

type stringArray struct {
	pointers **C.char
	length   int
}

func (s *stringArray) items() []*C.char {
	if s.pointers == nil {
		return nil
	}
	return unsafe.Slice(s.pointers, s.length)
}

func newStringArray(values []string) (*stringArray, error) {
	if len(values) == 0 {
		return &stringArray{}, nil
	}
	if uint64(len(values)) > uint64(^uintptr(0))/uint64(unsafe.Sizeof(uintptr(0))) {
		return nil, errors.New("cscandir: too many filter patterns")
	}
	pointers := C.calloc(C.size_t(len(values)), C.size_t(unsafe.Sizeof(uintptr(0))))
	if pointers == nil {
		return nil, errors.New("cscandir: out of memory allocating filter array")
	}
	s := &stringArray{pointers: (**C.char)(pointers), length: len(values)}
	for i, value := range values {
		if strings.IndexByte(value, 0) >= 0 {
			s.free()
			return nil, &Error{Code: ErrCodeNulByte, Msg: fmt.Sprintf("filter string %d contains a NUL byte", i)}
		}
		if !utf8.ValidString(value) {
			s.free()
			return nil, &Error{Code: ErrCodeInvalidUTF8, Msg: fmt.Sprintf("filter string %d is not valid UTF-8", i)}
		}
		c := C.CString(value)
		if c == nil {
			s.free()
			return nil, errors.New("cscandir: out of memory allocating filter string")
		}
		s.items()[i] = c
	}
	return s, nil
}

func (s *stringArray) free() {
	if s.pointers == nil {
		return
	}
	for i, item := range s.items() {
		if item != nil {
			C.free(unsafe.Pointer(item))
		}
		s.items()[i] = nil
	}
	C.free(unsafe.Pointer(s.pointers))
	s.pointers = nil
	s.length = 0
}

func (o *Options) cOptions() (C.cscandir_options, func(), error) {
	var native C.cscandir_options
	C.cscandir_options_init(&native)
	if o == nil {
		return native, func() {}, nil
	}
	maxSize := uint64(^uintptr(0))
	if o.MaxDepth > maxSize || o.MaxFileCnt > maxSize {
		return native, nil, &Error{Code: ErrCodeInvalidArgument, Msg: "depth or file limit exceeds the platform size limit"}
	}
	if o.ReturnType != ReturnTypeBase && o.ReturnType != ReturnTypeExt {
		return native, nil, &Error{Code: ErrCodeInvalidArgument, Msg: "return type must be ReturnTypeBase or ReturnTypeExt"}
	}
	native.sorted = boolToByte(o.Sorted)
	native.skip_hidden = boolToByte(o.SkipHidden)
	native.max_depth = C.size_t(o.MaxDepth)
	native.max_file_cnt = C.size_t(o.MaxFileCnt)
	native.case_sensitive = boolToByte(o.CaseSensitive)
	native.follow_links = boolToByte(o.FollowLinks)
	native.return_type = C.uint32_t(o.ReturnType)

	arrays := make([]*stringArray, 0, 4)
	cleanup := func() {
		for _, array := range arrays {
			array.free()
		}
	}
	newArray := func(values []string) (*stringArray, error) {
		array, err := newStringArray(values)
		if err != nil {
			return nil, err
		}
		arrays = append(arrays, array)
		return array, nil
	}
	dirInclude, err := newArray(o.DirInclude)
	if err != nil {
		cleanup()
		return native, nil, err
	}
	dirExclude, err := newArray(o.DirExclude)
	if err != nil {
		cleanup()
		return native, nil, err
	}
	fileInclude, err := newArray(o.FileInclude)
	if err != nil {
		cleanup()
		return native, nil, err
	}
	fileExclude, err := newArray(o.FileExclude)
	if err != nil {
		cleanup()
		return native, nil, err
	}
	native.dir_include, native.dir_include_len = dirInclude.pointers, C.size_t(dirInclude.length)
	native.dir_exclude, native.dir_exclude_len = dirExclude.pointers, C.size_t(dirExclude.length)
	native.file_include, native.file_include_len = fileInclude.pointers, C.size_t(fileInclude.length)
	native.file_exclude, native.file_exclude_len = fileExclude.pointers, C.size_t(fileExclude.length)
	return native, cleanup, nil
}

func boolToByte(value bool) C.uint8_t {
	if value {
		return 1
	}
	return 0
}

type errorSlot struct{ value C.cscandir_error }

func newErrorSlot() *errorSlot { return &errorSlot{} }

func (slot *errorSlot) release(code C.int32_t) error {
	defer C.cscandir_free_error(&slot.value)
	if code == C.CSCANDIR_OK {
		return nil
	}
	message := "unknown error"
	if slot.value.message != nil {
		message = C.GoString(slot.value.message)
	}
	return &Error{Code: int32(code), Msg: message}
}

func toGoString(list *C.cscandir_string_list) []string {
	if list == nil || list.len == 0 {
		return nil
	}
	items := unsafe.Slice(list.items, int(list.len))
	out := make([]string, len(items))
	for i, item := range items {
		if item != nil {
			out[i] = C.GoString(item)
		}
	}
	return out
}

func toGoStatistics(stats *C.cscandir_statistics) Statistics {
	return Statistics{
		Dirs: int32(stats.dirs), Files: int32(stats.files), Slinks: int32(stats.slinks),
		Hlinks: int32(stats.hlinks), Devices: int32(stats.devices), Pipes: int32(stats.pipes),
		Size: uint64(stats.size), Usage: uint64(stats.usage), Duration: float64(stats.duration),
	}
}

func toGoEntry(entry *C.cscandir_entry) Entry {
	out := Entry{
		IsSymlink: entry.is_symlink != 0, IsDir: entry.is_dir != 0, IsFile: entry.is_file != 0,
		Ctime: float64(entry.ctime), Mtime: float64(entry.mtime), Atime: float64(entry.atime),
		Size: uint64(entry.size), HasExt: entry.has_ext != 0,
		Mode: uint32(entry.mode), Ino: uint64(entry.ino), Dev: uint64(entry.dev),
		Nlink: uint64(entry.nlink), Blksize: uint64(entry.blksize), Blocks: uint64(entry.blocks),
		UID: uint32(entry.uid), GID: uint32(entry.gid), Rdev: uint64(entry.rdev),
	}
	if entry.path != nil {
		out.Path = C.GoString(entry.path)
	}
	return out
}

func validateRoot(root string) error {
	if strings.IndexByte(root, 0) >= 0 {
		return &Error{Code: ErrCodeNulByte, Msg: "root path contains a NUL byte"}
	}
	if !utf8.ValidString(root) {
		return &Error{Code: ErrCodeInvalidUTF8, Msg: "root path is not valid UTF-8"}
	}
	return nil
}

// Collect scans root and returns its entries and traversal errors.
// A nil options pointer selects the native library defaults.
func Collect(root string, options *Options) (*CollectResult, error) {
	if err := validateRoot(root); err != nil {
		return nil, err
	}
	native, release, err := options.cOptions()
	if err != nil {
		return nil, err
	}
	defer release()
	cRoot := C.CString(root)
	defer C.free(unsafe.Pointer(cRoot))
	var entries C.cscandir_entry_list
	var errs C.cscandir_string_list
	slot := newErrorSlot()
	start := time.Now()
	code := C.cscandir_collect(cRoot, &native, &entries, &errs, &slot.value)
	duration := time.Since(start).Seconds()
	runtime.KeepAlive(cRoot)
	runtime.KeepAlive(&native)
	defer C.cscandir_free_entry_list(&entries)
	defer C.cscandir_free_string_list(&errs)
	if err := slot.release(code); err != nil {
		return nil, err
	}
	goErrors := toGoString(&errs)
	result := &CollectResult{Errors: goErrors}
	if entries.len > 0 {
		list := unsafe.Slice(entries.entries, int(entries.len))
		result.Entries = make([]Entry, len(list))
		for i := range list {
			entry := toGoEntry(&list[i])
			result.Entries[i] = entry
		}
	}
	result.Statistics.Duration = duration
	return result, nil
}

// Count returns aggregate statistics without materializing entry results.
// Sorted has no effect on count. Traversal errors are returned in
// Statistics.Errors.
func Count(root string, options *Options) (Statistics, error) {
	if err := validateRoot(root); err != nil {
		return Statistics{}, err
	}
	native, release, err := options.cOptions()
	if err != nil {
		return Statistics{}, err
	}
	defer release()
	cRoot := C.CString(root)
	defer C.free(unsafe.Pointer(cRoot))
	var stats C.cscandir_statistics
	var errs C.cscandir_string_list
	slot := newErrorSlot()
	code := C.cscandir_count(cRoot, &native, &stats, &errs, &slot.value)
	runtime.KeepAlive(cRoot)
	runtime.KeepAlive(&native)
	defer C.cscandir_free_string_list(&errs)
	if err := slot.release(code); err != nil {
		return Statistics{}, err
	}
	goStats := toGoStatistics(&stats)
	goStats.Errors = toGoString(&errs)
	return goStats, nil
}

// Walk groups Collect results by their parent directory. The current C ABI
// has no native walk endpoint, so this is an in-Go grouping operation over a
// Collect result and therefore has the same memory cost as Collect.
func Walk(root string, options *Options) (*WalkResult, error) {
	collected, err := Collect(root, options)
	if err != nil {
		return nil, err
	}
	result := &WalkResult{Statistics: collected.Statistics}
	byDir := make(map[string]*WalkEntry)
	rootName := filepath.Base(filepath.Clean(root))
	if abs, absErr := filepath.Abs(root); absErr == nil {
		rootName = filepath.Base(filepath.Clean(abs))
	}
	for i, entry := range collected.Entries {
		if i == 0 && entry.IsDir && entry.Path == rootName {
			// Collect's synthetic root entry is not part of Walk's child TOCs.
			byDir[""] = &WalkEntry{}
			continue
		}
		parent, name := path.Dir(entry.Path), path.Base(entry.Path)
		if parent == "." {
			parent = ""
		}
		joined := name
		if parent != "" {
			joined = parent + "/" + name
		}
		walkEntry := byDir[parent]
		if walkEntry == nil {
			walkEntry = &WalkEntry{Dir: parent}
			byDir[parent] = walkEntry
		}
		if entry.IsDir {
			if _, exists := byDir[joined]; !exists {
				byDir[joined] = &WalkEntry{Dir: joined}
			}
		}
		if entry.IsSymlink {
			walkEntry.Toc.Symlinks = append(walkEntry.Toc.Symlinks, name)
			result.Toc.Symlinks = append(result.Toc.Symlinks, joined)
		} else if entry.IsDir {
			walkEntry.Toc.Dirs = append(walkEntry.Toc.Dirs, name)
			result.Toc.Dirs = append(result.Toc.Dirs, joined)
		} else if entry.IsFile {
			walkEntry.Toc.Files = append(walkEntry.Toc.Files, name)
			result.Toc.Files = append(result.Toc.Files, joined)
		} else {
			walkEntry.Toc.Other = append(walkEntry.Toc.Other, name)
			result.Toc.Other = append(result.Toc.Other, joined)
		}
	}
	for _, scanErr := range collected.Errors {
		result.Toc.Errors = append(result.Toc.Errors, scanErr)
	}
	dirs := make([]string, 0, len(byDir))
	for dir := range byDir {
		dirs = append(dirs, dir)
	}
	sort.Strings(dirs)
	for _, dir := range dirs {
		result.Roots = append(result.Roots, *byDir[dir])
	}
	return result, nil
}
