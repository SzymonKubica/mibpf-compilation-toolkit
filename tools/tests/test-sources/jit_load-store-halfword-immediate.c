// TEST_RESULT: 118
#include <stdint.h>
int load_store_word_immediate(void *ctx) {
    // The code below after compilation moves 100 into r1, then it stores
    // this value on the stack, loads it into r0 and adds 18 to it.
    // Thus it tests if the ldxh and stxh instructions are emitted correctly.
    volatile uint16_t x = 100;
    uint16_t y = 18;
    return x + y;
}
