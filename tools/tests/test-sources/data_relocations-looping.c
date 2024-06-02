// TEST_RESULT: 123
#include <stdint.h>
#include "helpers.h"

#define ITERATIONS 100000
uint32_t c = 0;
volatile uint32_t *ptr = &c;
// We add this string and return it so that the .rodata section is present
// in the binary and we need to check it when performing the memory access checks.
const char *rodata = "This a test read-only string";

int test_data_relocations()
{

    for (*ptr = 0; *ptr < ITERATIONS; (*ptr)++) {
    }

    return rodata[0];
}
