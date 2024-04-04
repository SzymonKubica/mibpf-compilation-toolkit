#include <stdint.h>
#include "helpers.h"

const int c = 123;
const int *ptr = &c;

// This test case checks whether data relocations are handled correctly by the
// VM. In this example, above we define a constant value 123 and a constant
// pointer to that value. The difficulty is that before loading the program into
// memory we don't know the address of the variable c, thus we cannot assign
// it to the variable ptr. Because of this, we need a relocation mechanism that
// runs at load time and fills in the correct addresses.
int test_data_relocations()
{
    bpf_printf("The two addresses below should be equal:\n");
    bpf_printf("ptr value: %p\n", ptr);
    bpf_printf("address of c: %p\n", &c);

    bpf_printf("We now dereference ptr and expect to get c: %d\n", *ptr);

    // We return the value of c accessed through ptr from the program.
    // This is done so that the testsuite can check whether the value is correct.
    return *ptr;
}
