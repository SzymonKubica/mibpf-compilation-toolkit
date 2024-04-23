// TEST_RESULT: 829540382
#include "helpers.h"

// Fletcher 32 checksum algorithm taken from:
// https://en.wikipedia.org/wiki/Fletcher%27s_checksum#:~:text=uint32_t%20fletcher32(const%20uint16_t%20*data%2C%20size_t%20len)
uint32_t fletcher32_checksum(void *ctx)
{
    volatile uint16_t data[] = {0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0};

    uint32_t len = (7 + 1) & ~1; /* Round up len to words */

    uint32_t c0 = 0;
    uint32_t c1 = 0;

    for (c0 = c1 = 0; len > 0;) {
        uint32_t blocklen = len;
        if (blocklen > 360 * 2) {
            blocklen = 360 * 2;
        }
        len -= blocklen;
        do {
            c0 = c0 + (*data)++;
            c1 = c1 + c0;
        } while ((blocklen -= 2));

        c0 = c0 % 65535;
        c1 = c1 % 65535;
    }
    uint32_t checksum = (c1 << 16 | c0);
    return checksum;
}
