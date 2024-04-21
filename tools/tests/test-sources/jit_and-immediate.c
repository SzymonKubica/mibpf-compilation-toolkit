// TEST_RESULT: 4
#include <stdint.h>
int and_immediate(void *ctx) {
    volatile int x = 0b1101;
    return x & 0b0110;
}
