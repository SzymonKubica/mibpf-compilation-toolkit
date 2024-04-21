// TEST_RESULT: 52
#include <stdint.h>
int lsl_immediate(void *ctx) {
    volatile int x = 13;
    return x << 2;
}
