#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "include/cscandir.h"

// File: scandir_collect.c
//
// Run scandir and collect to get final results.
int main(void) {
  cscandir_options *options = malloc(sizeof(cscandir_options));
  cscandir_options_init(options);
  const char *root_path = "/usr";
  cscandir_entry_list *out_entries = malloc(sizeof(cscandir_entry_list));
  cscandir_string_list *out_errors = malloc(sizeof(cscandir_string_list));
  cscandir_error *out_error = malloc(sizeof(cscandir_error));
  int32_t result =
      cscandir_collect(root_path, options, out_entries, out_errors, out_error);
  printf("result=%d\n", result);
  if (out_error->code != 0) {
    printf("error(%d)=%s\n", out_error->code, out_error->message);
  }
  if (out_errors->len > 0) {
    printf("%ld errors occurred:\n", out_errors->len);
    for (int i = 0; i < out_errors->len; i++) {
      printf("  %s\n", out_errors->items[i]);
    }
  }
  printf("entries.len=%ld\n", out_entries->len);
  for (int i = 0; i < out_entries->len; i++) {
    cscandir_entry *entry = &out_entries->entries[i];
    char typ = '?';
    if (entry->is_symlink)
      typ = 'S';
    if (entry->is_dir)
      typ = 'D';
    if (entry->is_file)
      typ = 'F';
    printf("entries[%d]: %c path=%s\n", i, typ, entry->path);
    printf("  ctime=%.2f\n", entry->ctime);
    printf("  size=%ld\n", entry->size);
  }
  cscandir_free_entry_list(out_entries);
  cscandir_free_string_list(out_errors);
  cscandir_free_error(out_error);
  free(options);
  return 0;
}
