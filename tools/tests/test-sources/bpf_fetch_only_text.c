// TEST_RESULT: 0
#include "helpers.h"

int test_bpf_fetch() {
    // First check the value to investigate that the storage
    // is empty (upon reruning the program it shouldn't be the case
    // as the number should have been written into the storage)
    bpf_store_global(0, 0);
    uint32_t value = 1234;
    bpf_fetch_global(0, &value);
    print("Value: %d\n", value);

    return value;
}
