// TEST_RESULT: 123
#include "helpers.h"
#include <stdint.h>

#define TEMP_DATA_START 0
#define TEMP_DATA_PTR 5
#define TEMP_STORAGE_SLOTS 4
int helper_call(void *ctx) {
    bpf_store_global(0, 0);
    uint32_t temp = 123;
    for (uint32_t i = 1; i < TEMP_STORAGE_SLOTS; i++) {
        uint32_t old_temp = 0;
        uint32_t offset = (0 + i) % TEMP_STORAGE_SLOTS;
        bpf_fetch_global(TEMP_DATA_START + offset, &old_temp);
        // We need to fill in empty values
        if (old_temp == 0) {
            bpf_store_global(TEMP_DATA_START + offset, temp);
        }
    }
    uint32_t result = 0;
    bpf_fetch_global(2, &result);
    return result;
}
