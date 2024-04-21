// TEST_RESULT: 27
#include <stdint.h>
int xor_reg(void *ctx) {
    volatile int x = 0b10101;
    volatile int z = 0b01110;
    return x ^ z;  //0b11011 = 27
}
