// TEST_RESULT: 8
#include <stdint.h>
int lsr_reg(void *ctx) {
    volatile uint32_t x = 64;
    volatile uint32_t z = 3;
    return x >> z;
}
