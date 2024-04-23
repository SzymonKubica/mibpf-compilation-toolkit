// TEST_RESULT: 1
#include <stdint.h>
int mod_immediate(void *ctx) {
    volatile uint16_t x = 50;
    return x % 7;
}
