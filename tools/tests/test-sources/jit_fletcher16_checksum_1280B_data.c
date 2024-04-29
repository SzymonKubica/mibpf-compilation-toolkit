// TEST_RESULT: 32742
#include "helpers.h"
#include <stdint.h>

// A random 1280B string
const char DATA[] =
    "46WgyN33S3oADXpVVIMX1ki2aMcO7fi8SN5HqvDtSJ6jqA96oHAKptpcAyxhVk4y"
    "2qlIEQB4YqErDyXUwMVJnOJEFzrHT0MC2RuOcY9tLCImE7OXyAU7opoXfKmkw8e6"
    "Q8Qm6wNAD7DHsBLYexQzXe2WDwADWaz6mENTwXqF6ZecRo2IyU9u93KFD3meVeIC"
    "fDezW9OeqLIDjwQ7FnOGwjSEeCZAqlpXACKmw3G2lsMHhGm44pygbapiYvBrfCgG"
    "UBNLhGdlUt9Hk0dCuBwAZjLu0pAf0ddJNicky8dUT9Zo6JNKkbrfuTU6cCfHe2nQ"
    "vZKGgfVQPuqoz4ahGJthZjUWsdXzREJSHmJIWvnFmarMd84mPQNKKqTH4kJMmy8c"
    "TMDyl5Gf81oscb2yFV7O8JizXETfnuvx5p0UqfzPr7E5AkRnbTd4m8135Vo4oVNH"
    "iKCE2HAdS6KPUUPLeMIJm7JfMx1a1bkchrkzu9EkO9CuYrPGsN2CMRIxuckPpK2q"
    "Iys8mm3oayC1z1sjZdboQDNR9oENO509932Zz0hA1ZjVWUvVzWc9cBbtabIFlHCs"
    "kkhKsjyydGJ6bddk0gSLzcseoGsaWgOfVhN4K9oysNZbs469FAdPBoTukToFaz7K"
    "46WgyN33S3oADXpVVIMX1ki2aMcO7fi8SN5HqvDtSJ6jqA96oHAKptpcAyxhVk4y"
    "2qlIEQB4YqErDyXUwMVJnOJEFzrHT0MC2RuOcY9tLCImE7OXyAU7opoXfKmkw8e6"
    "Q8Qm6wNAD7DHsBLYexQzXe2WDwADWaz6mENTwXqF6ZecRo2IyU9u93KFD3meVeIC"
    "fDezW9OeqLIDjwQ7FnOGwjSEeCZAqlpXACKmw3G2lsMHhGm44pygbapiYvBrfCgG"
    "UBNLhGdlUt9Hk0dCuBwAZjLu0pAf0ddJNicky8dUT9Zo6JNKkbrfuTU6cCfHe2nQ"
    "vZKGgfVQPuqoz4ahGJthZjUWsdXzREJSHmJIWvnFmarMd84mPQNKKqTH4kJMmy8c"
    "TMDyl5Gf81oscb2yFV7O8JizXETfnuvx5p0UqfzPr7E5AkRnbTd4m8135Vo4oVNH"
    "iKCE2HAdS6KPUUPLeMIJm7JfMx1a1bkchrkzu9EkO9CuYrPGsN2CMRIxuckPpK2q"
    "Iys8mm3oayC1z1sjZdboQDNR9oENO509932Zz0hA1ZjVWUvVzWc9cBbtabIFlHCs"
    "kkhKsjyydGJ6bddk0gSLzcseoGsaWgOfVhN4K9oysNZbs469FAdPBoTukToFaz7K";

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
