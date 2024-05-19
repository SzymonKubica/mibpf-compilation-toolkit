#include "helpers.h"

#define NO_INPUT 4
#define INTERVAL 50
int test_keypad() {

    uint32_t start = bpf_ztimer_now();

    while (1) {
        bpf_ztimer_periodic_wakeup(&start, INTERVAL);
        uint32_t x = bpf_keypad_get_input(2);
        if (x != NO_INPUT) {
          bpf_printf("You pressed: %d\n", x);
        }
    }
}
