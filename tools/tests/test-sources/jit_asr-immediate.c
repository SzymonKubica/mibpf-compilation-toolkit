// TEST_RESULT: -32
#include <stdint.h>
int asr_immediate(void *ctx) {
    volatile int8_t x = -64;
    return x >> 1;
}
