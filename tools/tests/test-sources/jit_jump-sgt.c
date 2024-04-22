// TEST_RESULT: 20
#include <stdint.h>
int jump_sgt(void *ctx) {
    // We need to use 16 bit ints here, otherwise lddw is used which is
    // not yet implemented
    volatile int16_t x = 100;
    if (x > -100) {
        return 123;
    }
    return 20;
}
