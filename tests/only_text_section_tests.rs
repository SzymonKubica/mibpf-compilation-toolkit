mod common;

use common::{test_execution, test_execution_accessing_coap_pkt};
use internal_representation::BinaryFileLayout;

// This module tests the VM executions with the OnlyTextSection binary file layout.
// This means that the program that the VM gets to execute contains only the
// .text section stripped off directly from the compiled ELF file. Because
// of this, the program isn't able to access .rodata / .data sections and
// so we need to use workarounds to access format strings when printing logs
// to the console.
//
// Advantages of this approach:
//  - simplicity (no preprocessing required)
//  - compatibility with the original version of rbpf VM
//  - small binary size
//
// Limitations:
//  - accessing format strings is limited (they need to be loaded onto the VM stack)
//  - no support for data relocations
//  - no support for pc-relative function calls (need to use inlined functions)
//  - no support for accessing CoAP packets (because default interpreter doesn't
//    allow for accessing those memory regions)

#[tokio::test]
async fn printf() {
    test_only_text_section("printf_only_text.c").await;
}

#[tokio::test]
async fn bpf_fetch() {
    test_only_text_section("bpf_fetch_only_text.c").await;
}

#[tokio::test]
async fn bpf_store() {
    test_only_text_section("bpf_store_only_text.c").await;
}

#[tokio::test]
async fn bpf_fmt_s16_dfp() {
    test_only_text_section("bpf_fmt_s16_dfp_only_text.c").await;
}

#[tokio::test]
async fn bpf_fmt_u32_dec() {
    test_only_text_section("bpf_fmt_u32_dec_only_text.c").await;
}

#[tokio::test]
async fn inlined_calls() {
    test_only_text_section("inlined_calls_only_text.c").await;
}

// Accessing CoAP packet buffers involves accessing memory that is outside of
// the specified regions that can be accessed by the default implementation of
// the interpreter in rbpf. Because of this, the test fails. It can be fixed
// by adapting the allowed memory regions
#[ignore]
#[tokio::test]
async fn gcoap_response_format() {
    test_only_text_section_accessing_coap_pkt("gcoap_response_format_only_text.c").await;
}

/// Tests the execution model where the program bytecode contains only the .text
/// section which is extracted directly from the object file. This format was
/// originally used by the rbpf VM. The advantage of this apporach is that the
/// size of the resulting binary is minimal, however we lose access to .data and
/// .rodata sections. The intended use case of this binary file layout is for
/// extremely short scripts that need to minimise the memory required to load
/// the program into the VM.
///
/// These tests also use the original version of the rbpf VM interpreter.
async fn test_only_text_section(test_program: &str) {
    test_execution(test_program, BinaryFileLayout::OnlyTextSection).await;
}

async fn test_only_text_section_accessing_coap_pkt(test_program: &str) {
    test_execution_accessing_coap_pkt(test_program, BinaryFileLayout::OnlyTextSection).await;
}
