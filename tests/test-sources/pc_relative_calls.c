// TEST_RESULT: 12345
#include "helpers.h"
static int __attribute__((noinline)) helper_function_1();
static int __attribute__((noinline)) helper_function_2(int x);
/// Here we check if the VM supports PC-relative function calls that are
/// specified in the eBPF ISA. In order to get the compiler to generate the code
/// which calls these functions in this way, we need to make the function
/// `static` and annotate it with `__attribute__((noinline))` to prevent the
/// compiler from inlining it.
int test_pc_relative_function_call(void *ctx)
{

    int helper_exit_code = helper_function_1();
    bpf_printf("The first helper function returned: %d\n", helper_exit_code);

    int helper_exit_code_2 = helper_function_2(123);
    return helper_exit_code_2;
}

static int __attribute__((noinline)) helper_function_1()
{
    bpf_printf("Inside the first helper function\n");

    char fmt[] = "printf accepts up to 4 args: %d %d %d %d\n";
    bpf_printf(fmt, 5, 6, 7, 8);

    return 1;
}

static int __attribute__((noinline)) helper_function_2(int x)
{
    bpf_printf("Inside the second helper function\n");

    bpf_printf("The value of argument x is: %d\n", x);

    return 12345;
}
