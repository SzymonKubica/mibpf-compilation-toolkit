// TEST_RESULT: 24
#include <stdint.h>
int basic_add(void *ctx) {
    volatile int x = 8;
    return x * 3;
}
