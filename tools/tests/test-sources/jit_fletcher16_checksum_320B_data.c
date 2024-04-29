// TEST_RESULT: 32742
#include "helpers.h"
#include <stdint.h>

// A random 320B string
const char DATA[] =
    "46WgyN33S3oADXpVVIMX1ki2aMcO7fi8SN5HqvDtSJ6jqA96oHAKptpcAyxhVk4y"
    "2qlIEQB4YqErDyXUwMVJnOJEFzrHT0MC2RuOcY9tLCImE7OXyAU7opoXfKmkw8e6"
    "Q8Qm6wNAD7DHsBLYexQzXe2WDwADWaz6mENTwXqF6ZecRo2IyU9u93KFD3meVeIC"
    "fDezW9OeqLIDjwQ7FnOGwjSEeCZAqlpXACKmw3G2lsMHhGm44pygbapiYvBrfCgG"
    "UBNLhGdlUt9Hk0dCuBwAZjLu0pAf0ddJNicky8dUT9Zo6JNKkbrfuTU6cCfHe2nQ";

// Fletcher 16 checksum algorithm taken from:
// https://en.wikipedia.org/wiki/Fletcher%27s_checksum#:~:text=uint32_t%20fletcher32(const%20uint16_t%20*data%2C%20size_t%20len)
uint32_t fletcher_16(void *ctx)
{

    uint8_t *data = (uint8_t *)DATA;

    size_t len = (bpf_strlen(DATA) + 1) & ~1; /* Round up len to words */

    uint16_t sum1 = 0;
    uint16_t sum2 = 0;
    int index;

    for (index = 0; index < len; ++index) {
        sum1 = (sum1 + data[index]) % 255;
        sum2 = (sum2 + sum1) % 255;
    }

    return (sum2 << 8) | sum1;
}
