// TEST_RESULT: 123
#include <stdint.h>
int subtract_immediate(void *ctx) {
    volatile int x = 144;
    int y = 21;
    return x - y;
}
