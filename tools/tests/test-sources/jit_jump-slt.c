// TEST_RESULT: 20
#include <stdint.h>
int jump_slt(void *ctx) {
    volatile int16_t x = -100;
    if (x < -200) {
        return 123;
    }
    return 20;
}
