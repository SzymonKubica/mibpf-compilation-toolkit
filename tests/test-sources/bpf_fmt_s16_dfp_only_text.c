// TEST_RESULT: 5
#include <stdint.h>
#include "helpers.h"

// We test bpf_fmt_s16_dfp helper here by formating the value -12.5 into a buffer.
int test_bpf_fmt_s16_dfp(void *ctx)
{
    int16_t val2 = -125;
    // We also test the second helper here, for integers that need not be
    // unsigned.
    char buffer[] = "     ";

    print("Buffer before formatting: %s\n", buffer);

    // Write the integer to the buffer.
    int chars_written = bpf_fmt_s16_dfp(buffer, val2, -1);

    print("Buffer after formatting: %s\n", buffer);

    return chars_written;
}
