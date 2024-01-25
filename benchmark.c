#include <stdint.h>
#include <string.h>
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

/*
 * Discussion points: eBPF seems to be limited w.r.t storing strings on the stack
 * When I tried including the 360B long string in the function code directly as
 * a constant. There was an error with illegal memory accesses. It could be
 * because that string couldn't fit in the stack.
 *
 */

SEC(".main")
int fletcher_32(void *ctx)
{
    // Similarly to femtocontainers. The checksum algorithm is run on a 360B
    // string (i.e. it contains 360 characters.
    //
    char *message =
        "This is a test message for the Fletcher32 checksum algorithm.\n";

    uint16_t *data = (uint16_t *)message;

    bpf_trace_printk("", 20, *data);

    // Algorithm needs the length in words
    size_t len = strlen(message) / 2;
    bpf_trace_printk("", 20, len);

    uint32_t c0 = 0;
    uint32_t c1 = 0;

    /* We similarly solve for n > 0 and n * (n+1) / 2 * (2^16-1) < (2^32-1)
     * here.
     */
    /* On modern computers, using a 64-bit c0/c1 could allow a group size of
     * 23726746. */
    for (c0 = c1 = 0; len > 0;) {
        size_t blocklen = len;
        if (blocklen > 360 * 2) {
            blocklen = 360 * 2;
        }
        len -= blocklen;
        for (size_t i = 0; i <= blocklen; i += 2) {
            bpf_trace_printk("", 20, *data);
            char c = *data++;
            c0 = c0 + c;
            c1 = c1 + c0;
        }
        c0 = c0 % 65535;
        c1 = c1 % 65535;
    }

    uint32_t checksum = (c1 << 16 | c0);

    return checksum;
}
