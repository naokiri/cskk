#include "libcskk.h"
#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

/**
 * Shallow checking of libcskk.h
 * Use rust tests for meaningful integration tests.
 *
 * TODO: Add a little bit more for examples to show
 */
int main() {
  char *version = skk_library_get_version();
  printf("Testing version: v%s\n", version);
  skk_free_string(version);

  CskkDictionaryFfi *dict[1];
  dict[0] = skk_file_dict_new("./tests/data/SKK-JISYO.S", "euc-jp", false);
  CskkContext *context;

  context = skk_context_new(dict, 1);
  // context = skk_context_new(NULL, 0);
  //  0x0061 = latin small a
  CskkKeyEvent *key_event =
      skk_key_event_new_from_fcitx_keyevent(0x0061, 0, false);
  bool retval = skk_context_process_key_event(context, key_event);
  assert(retval);
  char *output = skk_context_poll_output(context);
  printf("%s\n", output);
  assert(strcmp(output, "あ") == 0);
  skk_free_string(output);

  skk_free_dictionary(dict[0]);
  skk_free_context(context);
}