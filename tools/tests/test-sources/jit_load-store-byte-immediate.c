// TEST_RESULT: 118
#include <stdint.h>
int load_store_byte_immediate(void *ctx) {
    volatile uint8_t x = 100;
    uint8_t y = 18;
    return x + y;
}
