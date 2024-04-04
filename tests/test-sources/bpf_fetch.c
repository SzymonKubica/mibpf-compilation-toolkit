
#include "helpers.h"

int test_bpf_fetch() {
    // First check the value to investigate that the storage
    // is empty (upon reruning the program it shouldn't be the case
    // as the number should have been written into the storage)
    uint32_t value = 1234;
    bpf_fetch_global(0, &value);
    bpf_printf("Value: %d\n", value);

    // Assuming noone has written into that storage index before, the value
    // fetched from the global storage should be 0
    return value;
}
