// TEST_RESULT: 118
#include <stdint.h>
int load_store_halfword_immediate(void *ctx) {
    // The code below after compilation moves 100 into r1, then it stores
    // this value on the stack, loads it into r0 and adds 18 to it.
    // Thus it tests if the ldxw and stxw instructions are emitted correctly.
    volatile uint32_t x = 100;
    uint32_t y = 18;
    return x + y;
}
