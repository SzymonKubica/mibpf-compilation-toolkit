// TEST_RESULT: 123
#include <stdint.h>
int basic_add(void *ctx) {
    volatile int x = 144;
    int y = 21;
    return x - y;
}
