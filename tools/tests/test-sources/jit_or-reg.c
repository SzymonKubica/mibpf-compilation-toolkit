// TEST_RESULT: 31
#include <stdint.h>
int or_reg(void *ctx) {
    volatile int x = 0b10101;
    volatile int z = 0b01010;
    return x | z;
}
