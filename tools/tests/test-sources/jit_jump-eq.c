// TEST_RESULT: 123
#include <stdint.h>
int jump_eq(void *ctx) {
    volatile uint32_t x = 100;
    if (x == 100) {
        return 123;
    }
    return 0;
}
