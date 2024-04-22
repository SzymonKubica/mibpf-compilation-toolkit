// TEST_RESULT: 123
#include <stdint.h>
int jump_le(void *ctx) {
    volatile uint32_t x = 100;
    if (x < 200) {
        return 123;
    }
    return 0;
}
