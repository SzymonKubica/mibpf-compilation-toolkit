// TEST_RESULT: 20
#include <stdint.h>
int jump_sge(void *ctx) {
    volatile int16_t x = -100;
    if (x >= 50) {
        return 123;
    }
    return 20;
}
