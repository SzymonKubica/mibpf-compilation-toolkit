// TEST_RESULT: 123
#include <stdint.h>
int jump_eq(void *ctx) {
    volatile uint32_t x = 100;
    volatile uint32_t test = 100;
    if (x == test) {
        return 123;
    }
    return 0;
}
