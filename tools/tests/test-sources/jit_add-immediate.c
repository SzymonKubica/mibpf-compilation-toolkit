// TEST_RESULT: 123
#include <stdint.h>
int add_immediate(void *ctx) {
    volatile int x = 100;
    int y = 23;
    return x + y;
}
