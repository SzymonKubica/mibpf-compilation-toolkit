// TEST_RESULT: 32
#include <stdint.h>
int lsl_reg(void *ctx) {
    volatile int x = 4;
    volatile int z = 3;
    return x << z;
}
