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
int fletcher_32(struct __sk_buff *skb)
{
    void *data = (void *)(long)skb->data;
    void *data_end = (void *)(long)skb->data_end;
    struct eth_hdr *eth = data;
    struct iphdr *iph = data + sizeof(*eth);
    struct tcphdr *tcp = data + sizeof(*eth) + sizeof(*iph);

    // Ensure that there is some data
    if (data + sizeof(*eth) + sizeof(*iph) + sizeof(*tcp) > data_end)
        return -1;

    uint8_t *payload = data + sizeof(*eth) + sizeof(*iph) + sizeof(*tcp);

    uint8_t length = *payload;
    bpf_trace_printk("", 20, length);
    for (int i = 1; i <= length; i++) {
        bpf_trace_printk("", 20, *(payload + i));
    }

    size_t len = 22;

    uint32_t c0 = 0;
    uint32_t c1 = 0;

    for (c0 = c1 = 0; len > 0;) {
        size_t blocklen = len;
        if (blocklen > 360 * 2) {
            blocklen = 360 * 2;
        }
        len -= blocklen;
        for (size_t i = 0; i <= blocklen; i += 2) {
            char c = *payload++;
            c0 = c0 + c;
            c1 = c1 + c0;
        }
        c0 = c0 % 65535;
        c1 = c1 % 65535;
    }

    uint32_t checksum = (c1 << 16 | c0);

    return checksum;
}
