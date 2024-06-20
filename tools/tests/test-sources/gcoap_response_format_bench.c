// TEST_RESULT: {"temperature": -12.3}
#include <stdint.h>
#include "helpers.h"

#define COAP_OPT_FINISH_PAYLOAD (0x0001)

typedef struct {
    uint32_t hdr_p;       /* ptr to raw packet */
    uint32_t payload_p;   /* ptr to payload    */
    uint32_t token_p;     /* ptr to token      */
    uint16_t payload_len; /* length of payload */
    uint16_t options_len; /* length of options */
} bpf_coap_pkt_t;

typedef struct __attribute__((packed)) {
    uint8_t ver_t_tkl;
    uint8_t code;
    uint16_t id;
} coap_hdr_t;

const unsigned SUCCESS_RESPONSE_CODE = (2 << 5) | 5;

#define HUMIDITY_STORAGE_INDEX 1

static int get_temperature();
/// This test checks whether the end-to-end functionality of executing eBPF
/// programs with access to incoming network packets works correctly. This
/// program simulates getting some reading from a sensor and then uses that
/// reading to format the response that is sent back to the client who requested
/// execution of this program.
int gcoap_response_format(bpf_coap_ctx_t *gcoap)
{
    bpf_coap_pkt_t *pkt = gcoap->pkt;
    int temperature = get_temperature();

    char fmt_buffer[5];

    // -1 means that there is one decimal point.
    size_t str_len = bpf_fmt_s16_dfp(fmt_buffer, temperature, -1);

    bpf_gcoap_resp_init(gcoap, SUCCESS_RESPONSE_CODE);

    // Check that the code has been written correctly
    coap_hdr_t *hdr = (coap_hdr_t *)(intptr_t)(pkt->hdr_p);

    // Adding format adds an option to the packet. We should expect the number
    // of options to increase by 1.
    bpf_coap_add_format(gcoap, 0);
    ssize_t pdu_len = bpf_coap_opt_finish(gcoap, COAP_OPT_FINISH_PAYLOAD);

    uint8_t *payload = (uint8_t *)(pkt->payload_p);

    if (pkt->payload_len >= str_len) {
        char fmt[] = "{\"temperature\": }";
        int start_len = 16;
        int end_len = 2;
        bpf_memcpy(payload, fmt, start_len);
        bpf_memcpy(payload + start_len, fmt_buffer, str_len);
        bpf_memcpy(payload + start_len + str_len, fmt + start_len, end_len);
        // It is very important that the programs modifying response packet
        // buffer return the correct length of the payload. This is because this
        // return value is then used by the server to determine which subsection
        // of the buffer was written to and needs to be sent back to the client.
        return pdu_len + str_len + start_len + end_len;
    }
    return 0;
}

// Returns the temperature in degrees Celsius with one decimal point.
// Represented as an integer (temperature * 10).
static int get_temperature() { return -123; }
