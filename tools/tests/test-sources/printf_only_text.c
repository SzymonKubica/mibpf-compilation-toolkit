// TEST_RESULT: 100
#include "helpers.h"

// This testcase checks if the printing functionality works when using the
// OnlyTextSection binary layout. In this approach, the binary that is sent to
// the device contains only the .text section. Because of this, we aren't able to
// access the constant strings that are contained in the .rodata / .rodata.str.1
// sections. A possible workaround is to explicitly declare the format strings
// as variables in the program. The reason this works is that the compiled eBPF
// instructions then load all chars of the format string onto the stack of the
// eBPF VM. This has a limitation where using too large format strings can
// cause the stack to overflow (as it only has 512 bytes).
//
// The motivation behind using this simple approach is that it requires minimal
// pre-processing before the vm can start executing (as the first instruction
// to execute will be at the beginning of the supplied .text section). The problem
// is that the binary size is slightly larger as the format strings are loaded
// onto the stack by using sequences of lddw instructions which inflate the binary.
// See the example below:
//
//    0:   18 01 00 00 25 64 20 25         lddw %r1,7216209592422786085
//    8:   00 00 00 00 64 20 25 64
//   10:   7b 1a f0 ff 00 00 00 00         stxdw [%r10-16],%r1
//   18:   18 01 00 00 72 67 73 3a         lddw %r1,2334031327234582386
//   20:   00 00 00 00 20 25 64 20
//   28:   7b 1a e8 ff 00 00 00 00         stxdw [%r10-24],%r1
//   30:   18 01 00 00 70 20 74 6f         lddw %r1,6998651134847230064
//   38:   00 00 00 00 20 34 20 61
//   40:   7b 1a e0 ff 00 00 00 00         stxdw [%r10-32],%r1
//   48:   18 01 00 00 63 63 65 70         lddw %r1,8439872645631402851
//   50:   00 00 00 00 74 73 20 75
//   58:   7b 1a d8 ff 00 00 00 00         stxdw [%r10-40],%r1
//   60:   18 01 00 00 70 72 69 6e         lddw %r1,6998706471188394608
//   68:   00 00 00 00 74 66 20 61
//   70:   7b 1a d0 ff 00 00 00 00         stxdw [%r10-48],%r1
//   78:   b7 01 00 00 0a 00 00 00         mov %r1,10
//
//   This the assembly that gets generated for the program below. It uses 5
//   load-double-word instructions to load the format string onto the stack.
//   One could argue that having this information in the .rodata section would
//   occupy less bytes. However in that case we need to handle relocations in
//   some way. Refer to the RawObjectFile or FemtoContainersHeader binary
//   handle relocations.
int test_printf(void *ctx)
{


    print("printf accepts up to 4 args: %d %d %d %d\n", 1, 2, 3, 4);

    // We can also use the helper directly, however in that case we need to
    // first declare the char[]
    char fmt[] = "printf accepts up to 4 args: %d %d %d %d\n";
    bpf_printf(fmt, 5, 6, 7, 8);

    char FMT[] = "printf accepts up to 4 args: %d %d %d %d\n";
    bpf_printf(FMT, 9, 10, 11, 12);

    // In order to make it less unwieldy, a print macro is introduced which
    // declares the format string explicitly for us:

    // After the latest fixes to the rodata section, direct use of the format
    // string is also possible
    print("Here is a number: %d\n", 10);
    print("Here is another number: %d\n", 12);
    return 0;
}
