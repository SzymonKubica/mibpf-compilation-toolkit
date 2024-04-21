// TEST_RESULT: 150
#include <stdint.h>
int multiply_reg(void *ctx) {
    volatile int x = 10;
    int y = 5;
    volatile int z = 3;
    int ret = x * y;
    return ret * z;
}
