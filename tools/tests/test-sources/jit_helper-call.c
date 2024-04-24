// TEST_RESULT: 1000
#include "helpers.h"
#include <stdint.h>
int helper_call(void *ctx) {
    uint32_t result = 1234;
    bpf_store_global(0, 1000);
    bpf_fetch_global(0, &result);
    return result;
}
