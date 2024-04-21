// TEST_RESULT: 15
#include <stdint.h>
int or_immediate(void *ctx) {
    volatile int x = 0b1100;
    return x | 0b0011;
}
