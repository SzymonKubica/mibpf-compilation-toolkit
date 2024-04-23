// TEST_RESULT: 7
#include <stdint.h>
int divide_reg(void *ctx) {
    volatile uint16_t x = 49;
    uint16_t y = 7;
    return x / y;
}
