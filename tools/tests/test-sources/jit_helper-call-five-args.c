// TEST_RESULT: 1234
#include "helpers.h"
#include <stdint.h>
const char fmt[] = "This is a test of three helper args: %d %d %d %d\n";
int helper_call(void *ctx) {
    uint32_t result = 1234;
    volatile char *fmt_reg = fmt;

    uint32_t arg1 = 10000;
    uint32_t arg2 = 20000;
    uint32_t arg3 = 30000;
    uint32_t arg4 = 40000;
    bpf_printf(fmt_reg, arg1, arg2, arg3, arg4);
    return result;
}
