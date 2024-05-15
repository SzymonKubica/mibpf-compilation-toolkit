// TEST_RESULT: 1
#include "helpers.h"
#include <stdint.h>
int helper_call(void *ctx) {
    bpf_store_global(0, 0);
    uint32_t result = 0;
    for (uint32_t i = 0; i <= 10; i++ ) {
        bpf_fetch_global(0, &result);
        if (result == 0) {
            bpf_store_global(0, i);
        }
    }
    bpf_fetch_global(0, &result);
    return result;
}
