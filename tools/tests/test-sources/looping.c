// TEST_RESULT: 32742
#include "helpers.h"
#include <stdint.h>

#define ITERATIONS 100000

/* This file checks a simple for-loop iteration to investigate the
 * root cause of the performance discrepancy between rbpf and femtocontainers.
 */
uint32_t looping(void *ctx)
{

    uint32_t sum = 0;
    for (volatile uint32_t i = 0; i < ITERATIONS; i++) {
        //sum = (sum + i) % 255;
    }

    return sum;
}
