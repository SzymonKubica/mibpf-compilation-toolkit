// TEST_RESULT: 16
#include <stdint.h>
int lsr_immediate(void *ctx) {
    volatile int x = 64;
    return x >> 2;
}
