// TEST_RESULT: 5050
#include "helpers.h"
#include <stdint.h>
int while_loop_test(void *ctx) {
    uint32_t result = 0;
    int16_t counter = 100;
    while (counter > 0) {
        result += counter;
        bpf_printf("Counting down: %d\n", counter);
        counter--;
    }
    return result;
}
