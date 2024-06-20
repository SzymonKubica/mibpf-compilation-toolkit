#include <stdint.h>
#include "helpers.h"

#define SHARED_KEY 0x50
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

#define TEMPERATURE_STORAGE_START 0
#define TEMPERATURE_STORAGE_END 10
const unsigned SUCCESS_RESPONSE_CODE = (2 << 5) | 5;


int gcoap_temperature(bpf_coap_ctx_t *gcoap)
{
    bpf_coap_pkt_t *pkt = gcoap->pkt;

    uint32_t temperature_data[10];
    uint32_t temperature_reading;
    for (uint32_t i = TEMPERATURE_STORAGE_START; i < TEMPERATURE_STORAGE_END; i++) {
        bpf_fetch_global(i, &temperature_reading);
        temperature_data[i - TEMPERATURE_STORAGE_START] = temperature_reading;
    }

    uint32_t sum_temperature = 0;
    for (uint32_t i = TEMPERATURE_STORAGE_START; i < TEMPERATURE_STORAGE_END; i++) {
        sum_temperature += temperature_data[i - TEMPERATURE_STORAGE_START];
    }

    uint32_t avg_temperature =
        sum_temperature / (TEMPERATURE_STORAGE_END - TEMPERATURE_STORAGE_START);

    char fmt_buffer[5];

    // -1 means that there is one decimal point.
    size_t str_len = bpf_fmt_s16_dfp(fmt_buffer, avg_temperature, -1);

    bpf_gcoap_resp_init(gcoap, SUCCESS_RESPONSE_CODE);

    // Check that the code has been written correctly
    coap_hdr_t *hdr = (coap_hdr_t *)(intptr_t)(pkt->hdr_p);

    // Adding format adds an option to the packet. We should expect the number
    // of options to increase by 1.
    bpf_coap_add_format(gcoap, 0);
    ssize_t pdu_len = bpf_coap_opt_finish(gcoap, COAP_OPT_FINISH_PAYLOAD);

    uint8_t *payload = (uint8_t *)(pkt->payload_p);

    if (pkt->payload_len >= str_len) {
        uint32_t start_len = 16;
        uint32_t end_len = 2;
        char fmt[] = "{\"temperature\": }";
        bpf_memcpy(payload, fmt, start_len);
        bpf_memcpy(payload + start_len, fmt_buffer, str_len);
        bpf_memcpy(payload + start_len + str_len, fmt + start_len, end_len);
        // It is very important that the programs modifying response packet
        // buffer return the correct length of the payload. This is because this
        // return value is then used by the server to determine which subsection
        // of the buffer was written to and needs to be sent back to the client.
        return pdu_len + str_len + start_len + end_len;
    }
    return -1;
}
