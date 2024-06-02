// TEST_RESULT: 123
#include <stdint.h>
#include "helpers.h"

#define ITERATIONS 100000
int c = 0;
volatile int *ptr = &c;

int test_data_relocations()
{
    for (*ptr = 0; *ptr < ITERATIONS; (*ptr)++) {
        //sum = (sum + i) % 255;
    }

    return 0;
}
