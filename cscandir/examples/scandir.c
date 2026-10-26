#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "cscandir.h"

int main(int argc, char **argv) {
  const char *root_path = ".";
  int have_root = 0;
  int extended = 0;
  for (int i = 1; i < argc; ++i) {
    if (strcmp(argv[i], "--ext") == 0) {
      extended = 1;
    } else if (!have_root) {
      root_path = argv[i];
      have_root = 1;
    } else {
      fprintf(stderr, "Usage: %s [ROOT] [--ext]\n", argv[0]);
      return EXIT_FAILURE;
    }
  }

  cscandir_options options;
  cscandir_options_init(&options);
  options.sorted = 1;
  if (extended) {
    options.return_type = CSCANDIR_RETURN_EXT;
  }

  cscandir_entry_list entries = {0};
  cscandir_string_list errors = {0};
  cscandir_error error = {0};
  int32_t result = cscandir_collect(root_path, &options, &entries, &errors, &error);
  if (result != CSCANDIR_OK) {
    fprintf(stderr, "error(%" PRId32 ")=%s\n", error.code,
            error.message ? error.message : "scan failed");
    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);
    return EXIT_FAILURE;
  }

  printf("root=%s, metadata=%s, entries=%zu\n", root_path,
         extended ? "extended" : "base", entries.len);
  for (size_t i = 0; i < errors.len; ++i) {
    fprintf(stderr, "ERROR %s\n", errors.items[i]);
  }
  const int had_errors = errors.len != 0;
  for (size_t i = 0; i < entries.len; ++i) {
    const cscandir_entry *entry = &entries.entries[i];
    const char type = entry->is_symlink ? 'S' : entry->is_dir ? 'D' :
                      entry->is_file ? 'F' : '?';
    printf("%c %s size=%" PRIu64 " ctime=%.3f", type, entry->path,
           entry->size, entry->ctime);
    if (entry->has_ext) {
      printf(" mode=%" PRIu32 " ino=%" PRIu64 " uid=%" PRIu32 " gid=%" PRIu32,
             entry->mode, entry->ino, entry->uid, entry->gid);
    }
    putchar('\n');
  }

  cscandir_free_entry_list(&entries);
  cscandir_free_string_list(&errors);
  cscandir_free_error(&error);
  return had_errors ? EXIT_FAILURE : EXIT_SUCCESS;
}
