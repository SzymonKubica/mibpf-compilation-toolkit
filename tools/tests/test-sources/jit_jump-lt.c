// TEST_RESULT: 20
#include <stdint.h>
int jump_lt(void *ctx) {
    volatile uint32_t x = 100;
    if (x < 100) {
        return 123;
    }
    return 20;
}
