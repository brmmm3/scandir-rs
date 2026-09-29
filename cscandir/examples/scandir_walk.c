#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "cscandir.h"

// File: scandir_walk.c
//
// Run walk and print the per-directory breakdown of entries.
int main(int argc, char **argv) {
  const char *root_path = argc > 1 ? argv[1] : ".";

  cscandir_options options;
  cscandir_options_init(&options);

  cscandir_toc toc;
  cscandir_toc_init(&toc);
  cscandir_walk_entry_list entries;
  cscandir_walk_entry_list_init(&entries);
  cscandir_statistics stats;
  cscandir_error error;
  cscandir_error_init(&error);

  int32_t result =
      cscandir_walk(root_path, &options, &toc, &entries, &stats, &error);
  printf("result=%d\n", result);
  if (error.code != CSCANDIR_OK) {
    printf("error(%d)=%s\n", error.code, error.message);
  }

  printf("merged: dirs=%zu files=%zu symlinks=%zu other=%zu\n", toc.dirs.len,
         toc.files.len, toc.symlinks.len, toc.other.len);
  for (size_t i = 0; i < toc.errors.len; i++) {
    printf("  scan error: %s\n", toc.errors.items[i]);
  }

  printf("roots=%zu dirs=%d files=%d\n", entries.len, stats.dirs, stats.files);
  for (size_t i = 0; i < entries.len; i++) {
    cscandir_walk_entry *entry = &entries.entries[i];
    printf("%s: dirs=%zu files=%zu symlinks=%zu other=%zu\n", entry->path,
           entry->toc.dirs.len, entry->toc.files.len, entry->toc.symlinks.len,
           entry->toc.other.len);
  }

  cscandir_free_toc(&toc);
  cscandir_free_walk_entry_list(&entries);
  cscandir_free_error(&error);
  return 0;
}
