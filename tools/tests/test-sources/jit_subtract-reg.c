// TEST_RESULT: 77
#include <stdint.h>
int subtract_reg(void *ctx) {
    volatile int x = 100;
    int y = 18;
    volatile int z = 5;
    int ret = x - y;
    return ret - z;
}
