---
title: Shared API behavior
---

The bindings have different names and feature coverage. Consult the
[language guides](index.html#choose-a-language) for exact signatures. These
shared rules help when moving between interfaces.

## Choose the operation

- **Collect / Scandir** returns entries and metadata. It uses memory
  proportional to the number of returned entries.
- **Count** returns aggregate statistics without creating a result object for
  every entry. Prefer it when counts or size totals are all you need.
- **Walk** groups names by directory. Rust exposes the native walk
  implementation. C ABI wrappers that offer `walk` first collect the entire
  result and then group entries; that is not streaming. The current C++ binding
  exposes synchronous scan and count operations, not `Walk`.

Rust and Python support background workers and polling/intermediate results.
The C++ binding, C ABI, and wrappers built on it expose synchronous calls; a call
returns when scanning finishes.

## Defaults and options

Bindings generally expose sorting, hidden-entry handling, maximum depth and
entry count, directory/file glob filters, case sensitivity, link following, and
basic or extended metadata. Not every binding exposes every option.

- Hidden entries are included by default (`skip_hidden` is false).
- A zero depth or file-count limit means unlimited in the native APIs. Some
  wrappers use their own default sentinel, such as `None`, `SIZE_MAX`, or
  `Long.MAX_VALUE`.
- Directory filters match paths relative to the scan root; file filters match
  entry names. Validate filter syntax before starting a large scan.
- Sorting affects entry traversal/collection order. It does not change Count's
  aggregate values.
- Enable extended metadata only when needed. It can provide fields such as
  inode, device, ownership, block usage, and extended file-type information;
  platform support varies.

## Paths, results, and errors

Collected descendant paths are relative to the scan root. `collect` includes a
synthetic entry for the root itself, named with its basename; that entry may not
have stat metadata. Count statistics exclude the root directory.

An unreadable or otherwise problematic child can be reported as a per-entry
traversal error while the scan call itself succeeds. Fatal setup or scan errors
are reported separately: Rust returns `Result`, Go returns an `error`, Java
throws an exception, C returns a nonzero status, and Python raises an
exception. Check both the operation's fatal-error result and its per-entry
error collection.

## C ABI ownership

When calling C directly, initialize customized options with
`cscandir_options_init()` or pass `NULL` for defaults. Returned buffers belong
to the library: release them with `cscandir_free_entry_list()`,
`cscandir_free_string_list()`, and `cscandir_free_error()`. Do not free
individual elements, shallow-copy result structs for freeing, or reuse an
output struct without first freeing its previous contents. See the [C guide](c.html)
for a complete example.
