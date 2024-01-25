#include <stdint.h>
#include <string.h>
#include <linux/ip.h>
#include <linux/in.h>
#include <linux/tcp.h>
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

/*
 * Discussion points: eBPF seems to be limited w.r.t storing strings on the stack
 * When I tried including the 360B long string in the function code directly as
 * a constant. There was an error with illegal memory accesses. It could be
 * because that string couldn't fit in the stack.
 *
 */


#define ETH_ALEN 6
#define ETH_P_IP 0x0008 /* htons(0x0800) */
#define TCP_HDR_LEN 20

struct eth_hdr {
    unsigned char   h_dest[ETH_ALEN];
    unsigned char   h_source[ETH_ALEN];
    unsigned short  h_proto;
};

SEC(".main")
int fletcher_32(void *ctx)
{
    uint8_t *payload = (uint8_t *) ctx;
    for (int i = 0; i < 10; i++) {
        bpf_trace_printk("", 20, *(payload + 0x40 + i));
    }

    // Ensure that there is some data

    /*
    bpf_trace_printk("", 20, *payload);
    size_t len = 3;
    bpf_trace_printk("", 20, len);

    uint32_t c0 = 0;
    uint32_t c1 = 0;

    for (c0 = c1 = 0; len > 0;) {
        size_t blocklen = len;
        if (blocklen > 360 * 2) {
            blocklen = 360 * 2;
        }
        len -= blocklen;
        for (size_t i = 0; i <= blocklen; i += 2) {
            bpf_trace_printk("", 20, *payload);
            char c = *payload++;
            c0 = c0 + c;
            c1 = c1 + c0;
        }
        c0 = c0 % 65535;
        c1 = c1 % 65535;
    }

    uint32_t checksum = (c1 << 16 | c0);
    */

    return 0;
}
