// TEST_RESULT: 8
#include <stdint.h>
int lsr_reg(void *ctx) {
    volatile int x = 64;
    volatile int z = 3;
    return x >> z;
}
