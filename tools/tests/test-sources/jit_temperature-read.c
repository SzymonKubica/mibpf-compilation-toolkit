#include <stdint.h>
#include "helpers.h"

#define SAUL_SENSE_TEMP 130

const char FMT[] = "Sensor struct pointer: %x\n";
const char FMT2[] = "Temperature struct pointer: %x\n";

uint32_t temperature_read(void *ctx)
{
    // First we read the temperature
    bpf_saul_reg_t *dht_temp;
    dht_temp = bpf_saul_reg_find_type(SAUL_SENSE_TEMP);
    bpf_printf(FMT, dht_temp);

    uint32_t temperature_data;
    bpf_saul_read_temp(dht_temp, &temperature_data);
    return 0;
}
