// Package cscandir provides Go bindings for the scandir-rs directory scanner.
//
// It wraps the C ABI exported by the `cscandir` crate. Memory returned by the
// C layer is copied into Go values, so nothing has to be released by hand and
// results stay valid independently of the C library.
//
//	result, err := cscandir.Collect("/usr", &cscandir.Options{SkipHidden: true})
//	if err != nil {
//	    return err
//	}
//	for _, entry := range result.Entries {
//	    fmt.Println(entry.Path, entry.Size)
//	}
//
// The package requires cgo and links against libcscandir. By default it looks
// for the library in ../target/release relative to this package; override with
// the CGO_CFLAGS and CGO_LDFLAGS environment variables.
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
	"runtime"
	"unsafe"
)

// Error codes reported by the C library.
const (
	ErrCodeOK              = C.CSCANDIR_OK
	ErrCodeInvalidArgument = C.CSCANDIR_ERR_INVALID_ARGUMENT
	ErrCodeInvalidUTF8     = C.CSCANDIR_ERR_INVALID_UTF8
	ErrCodeScan            = C.CSCANDIR_ERR_SCAN
	ErrCodeNulByte         = C.CSCANDIR_ERR_NUL_BYTE
)

// ReturnType selects how much detail each entry carries.
type ReturnType uint32

const (
	// ReturnTypeBase reports path, type flags, timestamps and size.
	ReturnTypeBase ReturnType = C.CSCANDIR_RETURN_BASE
	// ReturnTypeExt additionally populates the extended stat fields.
	ReturnTypeExt ReturnType = C.CSCANDIR_RETURN_EXT
)

// Error is returned when a scan fails. Code carries the C error code and Path
// is set when the failure concerned a specific directory.
type Error struct {
	Code int32
	Msg  string
}

func (e *Error) Error() string {
	return fmt.Sprintf("cscandir: %s (code %d)", e.Msg, e.Code)
}

// Is reports whether the target is an *Error with the given code, which makes
// errors.Is(err, cscandir.ErrCodeScan) style checks possible.
func (e *Error) Is(target error) bool {
	var other *Error
	if !errors.As(target, &other) {
		return false
	}
	return other.Code == e.Code
}

// ErrInvalidArgument matches any error carrying ErrCodeInvalidArgument.
var ErrInvalidArgument error = &Error{Code: ErrCodeInvalidArgument, Msg: "invalid argument"}

// ErrScan matches any error carrying ErrCodeScan.
var ErrScan error = &Error{Code: ErrCodeScan, Msg: "scan failed"}

// Options configures a scan. The zero value is valid and matches the defaults
// used by scandir::Scandir.
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

// Statistics summarises a scan.
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
}

// Entry is a single directory entry.
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

	// HasExt reports whether the extended fields below were populated, which
	// requires ReturnTypeExt.
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

// Toc groups entry names by type.
type Toc struct {
	Dirs     []string
	Files    []string
	Symlinks []string
	Other    []string
	Errors   []string
}

// WalkEntry is one directory visited by Walk, with the entries it held. Dir is
// relative to the scan root and is empty for the root itself.
type WalkEntry struct {
	Dir string
	Toc Toc
}

// CollectResult is returned by Collect.
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

// ---------------------------------------------------------------------------
// Bridging helpers
// ---------------------------------------------------------------------------

// stringArray is a C array of NUL-terminated strings plus its length, as the C
// options struct expects. The underlying C strings are released by free.
type stringArray struct {
	pointers **C.char
	length   int
}

// items exposes the array as a Go slice so it can be indexed and ranged over.
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
	s := &stringArray{
		pointers: (**C.char)(C.malloc(C.size_t(len(values)) * C.size_t(unsafe.Sizeof(uintptr(0))))),
		length:   len(values),
	}
	allocated := 0
	for _, value := range values {
		c := C.CString(value)
		if c == nil {
			s.free()
			return nil, errors.New("cscandir: out of memory allocating filter")
		}
		s.items()[allocated] = c
		allocated++
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
			s.items()[i] = nil
		}
	}
	C.free(unsafe.Pointer(s.pointers))
	s.pointers = nil
	s.length = 0
}

// cOptions renders Options into the C representation, wiring up the filter
// arrays. The returned release function frees the filter strings.
func (o *Options) cOptions() (C.cscandir_options, func(), error) {
	var native C.cscandir_options
	C.cscandir_options_init(&native)

	if o == nil {
		return native, func() {}, nil
	}

	native.sorted = boolToByte(o.Sorted)
	native.skip_hidden = boolToByte(o.SkipHidden)
	native.max_depth = C.size_t(o.MaxDepth)
	native.max_file_cnt = C.size_t(o.MaxFileCnt)
	native.case_sensitive = boolToByte(o.CaseSensitive)
	native.follow_links = boolToByte(o.FollowLinks)
	native.return_type = C.uint32_t(o.ReturnType)

	// The filter arrays borrow memory owned by these strings, which must
	// outlive the C call that reads them.
	dirInclude, err := newStringArray(o.DirInclude)
	if err != nil {
		return native, func() {}, err
	}
	dirExclude, err := newStringArray(o.DirExclude)
	if err != nil {
		dirInclude.free()
		return native, func() {}, err
	}
	fileInclude, err := newStringArray(o.FileInclude)
	if err != nil {
		dirInclude.free()
		dirExclude.free()
		return native, func() {}, err
	}
	fileExclude, err := newStringArray(o.FileExclude)
	if err != nil {
		dirInclude.free()
		dirExclude.free()
		fileInclude.free()
		return native, func() {}, err
	}

	native.dir_include = dirInclude.pointers
	native.dir_include_len = C.size_t(dirInclude.length)
	native.dir_exclude = dirExclude.pointers
	native.dir_exclude_len = C.size_t(dirExclude.length)
	native.file_include = fileInclude.pointers
	native.file_include_len = C.size_t(fileInclude.length)
	native.file_exclude = fileExclude.pointers
	native.file_exclude_len = C.size_t(fileExclude.length)

	release := func() {
		dirInclude.free()
		dirExclude.free()
		fileInclude.free()
		fileExclude.free()
	}
	return native, release, nil
}

func boolToByte(value bool) C.uint8_t {
	if value {
		return 1
	}
	return 0
}

// errorSlot owns a C error out-struct for the duration of one call.
type errorSlot struct {
	value C.cscandir_error
}

func newErrorSlot() *errorSlot {
	slot := &errorSlot{}
	C.cscandir_error_init(&slot.value)
	return slot
}

// release reads the error out and returns it as a Go error, or nil on success.
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
	out := make([]string, 0, int(list.len))
	items := unsafe.Slice(list.items, int(list.len))
	for _, item := range items {
		if item == nil {
			out = append(out, "")
			continue
		}
		out = append(out, C.GoString(item))
	}
	return out
}

func toGoToc(toc *C.cscandir_toc) Toc {
	if toc == nil {
		return Toc{}
	}
	return Toc{
		Dirs:     toGoString(&toc.dirs),
		Files:    toGoString(&toc.files),
		Symlinks: toGoString(&toc.symlinks),
		Other:    toGoString(&toc.other),
		Errors:   toGoString(&toc.errors),
	}
}

func toGoStatistics(stats *C.cscandir_statistics) Statistics {
	if stats == nil {
		return Statistics{}
	}
	return Statistics{
		Dirs:     int32(stats.dirs),
		Files:    int32(stats.files),
		Slinks:   int32(stats.slinks),
		Hlinks:   int32(stats.hlinks),
		Devices:  int32(stats.devices),
		Pipes:    int32(stats.pipes),
		Size:     uint64(stats.size),
		Usage:    uint64(stats.usage),
		Duration: float64(stats.duration),
	}
}

func toGoEntry(entry *C.cscandir_entry) Entry {
	out := Entry{
		IsSymlink: entry.is_symlink != 0,
		IsDir:     entry.is_dir != 0,
		IsFile:    entry.is_file != 0,
		Ctime:     float64(entry.ctime),
		Mtime:     float64(entry.mtime),
		Atime:     float64(entry.atime),
		Size:      uint64(entry.size),
		HasExt:    entry.has_ext != 0,
		Mode:      uint32(entry.mode),
		Ino:       uint64(entry.ino),
		Dev:       uint64(entry.dev),
		Nlink:     uint64(entry.nlink),
		Blksize:   uint64(entry.blksize),
		Blocks:    uint64(entry.blocks),
		UID:       uint32(entry.uid),
		GID:       uint32(entry.gid),
		Rdev:      uint64(entry.rdev),
	}
	if entry.path != nil {
		out.Path = C.GoString(entry.path)
	}
	return out
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

// Collect scans root and returns an entry per result (scandir::Scandir).
//
// A nil options is valid and selects the library defaults.
func Collect(root string, options *Options) (*CollectResult, error) {
	native, release, err := options.cOptions()
	if err != nil {
		return nil, err
	}
	defer release()

	cRoot := C.CString(root)
	defer C.free(unsafe.Pointer(cRoot))

	var entries C.cscandir_entry_list
	C.cscandir_entry_list_init(&entries)
	defer C.cscandir_free_entry_list(&entries)

	var errs C.cscandir_string_list
	C.cscandir_string_list_init(&errs)
	defer C.cscandir_free_string_list(&errs)

	var stats C.cscandir_statistics
	slot := newErrorSlot()

	// The C call borrows cRoot and the filter arrays held by options, so keep
	// both alive until it returns.
	code := C.cscandir_collect(cRoot, &native, &entries, &errs, &stats, &slot.value)
	runtime.KeepAlive(cRoot)
	runtime.KeepAlive(&native)
	if err := slot.release(code); err != nil {
		return nil, err
	}

	result := &CollectResult{
		Entries:    make([]Entry, 0, int(entries.len)),
		Errors:     toGoString(&errs),
		Statistics: toGoStatistics(&stats),
	}
	if entries.len > 0 {
		list := unsafe.Slice(entries.entries, int(entries.len))
		for i := range list {
			result.Entries = append(result.Entries, toGoEntry(&list[i]))
		}
	}
	return result, nil
}

// Count returns scan statistics without materialising entries
// (scandir::Count). The Sorted option is ignored here, because Count has no
// such option.
func Count(root string, options *Options) (Statistics, error) {
	native, release, err := options.cOptions()
	if err != nil {
		return Statistics{}, err
	}
	defer release()

	cRoot := C.CString(root)
	defer C.free(unsafe.Pointer(cRoot))

	var stats C.cscandir_statistics
	slot := newErrorSlot()

	code := C.cscandir_count(cRoot, &native, &stats, nil, &slot.value)
	runtime.KeepAlive(cRoot)
	runtime.KeepAlive(&native)
	if err := slot.release(code); err != nil {
		return Statistics{}, err
	}
	return toGoStatistics(&stats), nil
}

// Walk groups entries by directory (scandir::Walk).
//
// Note that scandir::Walk defaults SkipHidden to true in Rust, while these
// options default to false to match Scandir and Count. Set it explicitly to
// reproduce the Rust default.
func Walk(root string, options *Options) (*WalkResult, error) {
	native, release, err := options.cOptions()
	if err != nil {
		return nil, err
	}
	defer release()

	cRoot := C.CString(root)
	defer C.free(unsafe.Pointer(cRoot))

	var toc C.cscandir_toc
	C.cscandir_toc_init(&toc)
	defer C.cscandir_free_toc(&toc)

	var roots C.cscandir_walk_entry_list
	C.cscandir_walk_entry_list_init(&roots)
	defer C.cscandir_free_walk_entry_list(&roots)

	var stats C.cscandir_statistics
	slot := newErrorSlot()

	code := C.cscandir_walk(cRoot, &native, &toc, &roots, &stats, &slot.value)
	runtime.KeepAlive(cRoot)
	runtime.KeepAlive(&native)
	if err := slot.release(code); err != nil {
		return nil, err
	}

	result := &WalkResult{
		Toc:        toGoToc(&toc),
		Roots:      make([]WalkEntry, 0, int(roots.len)),
		Statistics: toGoStatistics(&stats),
	}
	if roots.len > 0 {
		list := unsafe.Slice(roots.entries, int(roots.len))
		for i := range list {
			entry := WalkEntry{Toc: toGoToc(&list[i].toc)}
			if list[i].path != nil {
				entry.Dir = C.GoString(list[i].path)
			}
			result.Roots = append(result.Roots, entry)
		}
	}
	return result, nil
}
