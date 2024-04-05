// TEST_RESULT: 5
#include <stdint.h>
#include "helpers.h"

int test_fmt(void *ctx)
{

    uint32_t val = 12345;
    // We also test the second helper here, for integers that need not be
    // unsigned.
    char *buffer = "     ";

    bpf_printf("Buffer before formatting: %s\n", buffer);

    // Write the integer to the buffer.
    int chars_written = bpf_fmt_u32_dec(buffer, val);

    bpf_printf("Buffer after formatting: %s\n", buffer);

    return chars_written;
}
