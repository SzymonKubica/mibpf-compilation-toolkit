// TEST_RESULT: 1
#include <stdint.h>
int divide_reg(void *ctx) {
    volatile uint16_t x = 50;
    uint16_t y = 7;
    return x % y;
}
