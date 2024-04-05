// TEST_RESULT: 1234
#include <stdint.h>
#include "helpers.h"

const uint32_t STORAGE_INDEX = 1;

int test_bpf_store()
{
    // First check the value to investigate that the storage
    // is empty (upon reruning the program it shouldn't be the case
    // as the number should have been written into the storage)
    uint32_t value = 0;
    bpf_fetch_global(STORAGE_INDEX, &value);

    print("Initial value in the global storage at index %d: %d\n",
               STORAGE_INDEX, value);

    bpf_store_global(STORAGE_INDEX, 1234);

    bpf_fetch_global(STORAGE_INDEX, &value);

    print("Value after bpf_store_global: %d\n", value);

    return value;
}
