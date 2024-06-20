// TEST_RESULT: 0
#include "helpers.h"

// This string should go into the .rodata section
const char FMT[] = "i: %d, %d\n";

int test_printf(void *ctx)
{

    for (int i = 0; i < 30; i++) {
        bpf_printf(FMT, i, i);
    }

    return 0;
}
