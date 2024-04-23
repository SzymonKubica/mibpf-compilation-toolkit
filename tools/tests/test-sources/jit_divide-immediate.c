// TEST_RESULT: 7
#include <stdint.h>
int divide_immediate(void *ctx) {
    volatile uint16_t x = 49;
    return x / 7;
}
