#include <stdbool.h>
#include <assert.h>
#include "libcskk.h"


int main() {
    CskkContext *context;
    context = create_new_context();

    bool retval = skk_context_process_key_events(context, "a");
    assert(retval);
}