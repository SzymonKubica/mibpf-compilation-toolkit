// TEST_RESULT: 16
#include <stdint.h>
int asr_reg(void *ctx) {
    volatile int16_t a = -64;
    volatile int16_t b = 2;
    int16_t c = a >> b;
    return -1 * c;
}
