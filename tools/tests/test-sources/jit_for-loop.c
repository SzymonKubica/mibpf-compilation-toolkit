// TEST_RESULT: 100
#include "helpers.h"
#include <stdint.h>
int helper_call(void *ctx) {
    uint32_t result = 0;
    for (int16_t i = 0; i <= 1000; i++ ) {
        bpf_store_global(0, i);
    }
    bpf_fetch_global(0, &result);
    return result;
}
