// TEST_RESULT: 500500
#include "helpers.h"
#include <stdint.h>
int while_loop_test(void *ctx) {
    uint32_t result = 0;
    int16_t counter = 1000;
    while (counter > 0) {
        result += counter;
        bpf_printf("Counting down: %d\n", counter);
        counter--;
    }
    return result;
}
