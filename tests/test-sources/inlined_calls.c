// TEST_RESULT: 12345
#include "helpers.h"
inline int helper_function_1();
inline int helper_function_2(int x);


// This test checks if the VM supports inlined function calls. This should
// work on all types of VMs and binary layouts. This is because here the work
// is done by the compiler (as the function instructions are inlined at the
// call site). The disadvantage of this approach is larger binary size. An
// advantage is that it is compatible with the OnlyTextSection binary file
// layout and thus can be used with the vanilla implementation of the rbpr
// eBPF VM.
int test_inlined_calls(void *ctx)
{

    int helper_exit_code = helper_function_1();
    bpf_printf("The first helper function returned: %d\n", helper_exit_code);

    int helper_exit_code_2 = helper_function_2(123);
    return helper_exit_code_2;
}

inline int helper_function_1()
{
    bpf_printf("Inside the first helper function\n");

    char fmt[] = "printf accepts up to 4 args: %d %d %d %d\n";
    bpf_printf(fmt, 5, 6, 7, 8);

    return 1;
}

inline int helper_function_2(int x)
{
    bpf_printf("Inside the second helper function\n");

    bpf_printf("The value of argument x is: %d\n", x);

    return 12345;
}
