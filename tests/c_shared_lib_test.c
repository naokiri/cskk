#include <stdbool.h>
#include <assert.h>
#include "libcskk.h"

/**
 * Shallow checking of libcskk.h
 * Use rust tests for meaningful integration tests.
 *
 * TODO: Add a little bit more for examples to show
 */
int main() {
    CskkDictionary *dict[1];
    dict[0] = skk_file_dict_new("./tests/data/SKK-JISYO.S", "euc-jp");
    CskkContext *context;

    context = skk_context_new(dict, 1);

    bool retval = skk_context_process_key_events(context, "a");
    assert(retval);
    skk_context_free(context);
}