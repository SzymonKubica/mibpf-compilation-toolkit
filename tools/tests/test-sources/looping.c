// TEST_RESULT: 32742
#include "helpers.h"
#include <stdint.h>

#define ITERATIONS 100000

// We add this string and return it so that the .rodata section is present
// in the binary and we need to check it when performing the memory access
// checks.
const char *rodata = "This a test read-only string";

/* This file checks a simple for-loop iteration to investigate the
 * root cause of the performance discrepancy between rbpf and femtocontainers.
 */
uint32_t looping(void *ctx)
{
    for (volatile uint32_t i = 0; i < ITERATIONS; i++) {
    }

    return rodata[0];
}
