// TEST_RESULT: 829540382
#include "helpers.h"


// A random 320B string
const char DATA[] =
    "46WgyN33S3oADXpVVIMX1ki2aMcO7fi8SN5HqvDtSJ6jqA96oHAKptpcAyxhVk4y"
    "2qlIEQB4YqErDyXUwMVJnOJEFzrHT0MC2RuOcY9tLCImE7OXyAU7opoXfKmkw8e6"
    "Q8Qm6wNAD7DHsBLYexQzXe2WDwADWaz6mENTwXqF6ZecRo2IyU9u93KFD3meVeIC"
    "fDezW9OeqLIDjwQ7FnOGwjSEeCZAqlpXACKmw3G2lsMHhGm44pygbapiYvBrfCgG"
    "UBNLhGdlUt9Hk0dCuBwAZjLu0pAf0ddJNicky8dUT9Zo6JNKkbrfuTU6cCfHe2nQ";

// Fletcher 32 checksum algorithm taken from:
// https://en.wikipedia.org/wiki/Fletcher%27s_checksum#:~:text=uint32_t%20fletcher32(const%20uint16_t%20*data%2C%20size_t%20len)
uint32_t fletcher32_checksum()
{
    uint16_t *data = (uint16_t *)DATA;

    size_t len = (bpf_strlen(DATA) + 1) & ~1; /* Round up len to words */
    bpf_printf("Length of the checksummed data: %d\n", len);

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
    bpf_printf("Calculated the checksum: %u\n", checksum);
    return checksum;
}
