// TEST_RESULT: 7
#include <stdint.h>
int xor_immediate(void *ctx) {
    volatile int x = 0b010;
    return x ^ 0b101; // 0b111 = 7
}
