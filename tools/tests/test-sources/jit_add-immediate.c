// TEST_RESULT: 123
#include <stdint.h>
int basic_add(void *ctx) {
    volatile int x = 100;
    int y = 23;
    return x + y;
}
