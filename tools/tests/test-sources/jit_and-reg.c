// TEST_RESULT: 4
#include <stdint.h>
int add_reg(void *ctx) {
    volatile int x = 0b1111;
    int y = 0b1100;
    volatile int z = 0b111;
    int ret = x & y;
    return ret & z; // 0b100
}
