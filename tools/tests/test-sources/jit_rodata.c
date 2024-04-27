// TEST_RESULT: 5
#include <stdint.h>
#include "helpers.h"
char rodata[] = "hello";

int jit_rodata(void *ctx) {
    bpf_printf("%s world\n", rodata);
    uint16_t len = bpf_strlen(rodata);
    return len;
}
