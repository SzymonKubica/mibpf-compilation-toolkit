// TEST_RESULT: 21
#include "helpers.h"

int test_bpf_strlen()
{
    char str[] = "This is a test string";
    size_t len = bpf_strlen(str);

    bpf_printf("Length of the string: %d\n", len);

    return len;
}
