#include "helpers.h"

const char *DATA = "This is the string that will be checksummed.";

// Fletcher 32 checksum algorithm taken from:
// https://en.wikipedia.org/wiki/Fletcher%27s_checksum#:~:text=uint32_t%20fletcher32(const%20uint16_t%20*data%2C%20size_t%20len)
uint32_t fletcher32_checksum()
{
    uint16_t *data = (uint16_t *)DATA;
    size_t data_len = 44;

    size_t len = (bpf_strlen(DATA) + 1) & ~1; /* Round up len to words */

    uint32_t c0 = 0;
    uint32_t c1 = 0;

    for (c0 = c1 = 0; len > 0;) {
        uint32_t blocklen = len;
        if (blocklen > 360 * 2) {
            blocklen = 360 * 2;
        }
        len -= blocklen;
        do {
            c0 = c0 + *data++;
            c1 = c1 + c0;
        } while ((blocklen -= 2));

        c0 = c0 % 65535;
        c1 = c1 % 65535;
    }
    uint32_t checksum = (c1 << 16 | c0);
    bpf_printf("Calculated the checksum: %d", checksum);
    return checksum;
}
