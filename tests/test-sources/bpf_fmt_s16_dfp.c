// TEST_RESULT: 5
#include <stdint.h>
#include "helpers.h"

int test_fmt(void *ctx)
{

    int16_t val2 = -125;

    // We also test the second helper here, for integers that need not be
    // unsigned.
    char *buffer = "     ";

    bpf_printf("Buffer before formatting: %s\n", buffer);

    // Write the integer to the buffer.
    int chars_written = bpf_fmt_s16_dfp(buffer, val2, -1);

    bpf_printf("Buffer after formatting: %s\n", buffer);

    return chars_written;
}
