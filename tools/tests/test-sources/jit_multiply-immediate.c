// TEST_RESULT: 24
#include <stdint.h>
int miltiply_immediate(void *ctx) {
    volatile int x = 8;
    return x * 3;
}
