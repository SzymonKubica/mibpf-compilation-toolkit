mod common;

use common::{
    test_execution, test_execution_accessing_coap_pkt,
    test_execution_femtocontainer_vm, test_execution_accessing_coap_pkt_femtocontainer_vm,
};
use micro_bpf_tools::load_env;

use micro_bpf_common::BinaryFileLayout;

// This module contains end-to-end integration tests of the compile-upload-
// execute workflow of the eBPF programs on microcontrollers. It is recommended
// that the tests are run using a native RIOT instance running on the host
// desktop machine.
//
// The tests are set up in a way that each test file contains the expected return
// value on the first line in the source file. This testsuite extracts that information
// and compares it to the actual output returned in the response from the RIOT
// instance running the micro_bpf server.
//
// The tests in this module are specifically for the `FemtoContainersHeader`
// binary file layout, which should be supported by both the rbpf VM implementation
// when using the `FemtoContainersHeader` interpreter version and the original
// version of the VM proposed by FemtoContainers. Because of this, the tests
// in this file always run on both implementations of the VM and can be used as
// a regression that the new functionality is fully backwards-compatible.
//
// Some notable limitations of the Femto-Containers VM implementation:
//
// - no support for PC relative calls
// - a smaller set of supported helpers (no bpf_strlen, gpio, saul ...)

#[tokio::test]
async fn printf() {
    test_femtocontainers_header("printf.c").await;
}

#[tokio::test]
async fn bpf_fetch() {
    test_femtocontainers_header("bpf_fetch.c").await;
}

#[tokio::test]
async fn bpf_store() {
    test_femtocontainers_header("bpf_store.c").await;
}

#[tokio::test]
async fn bpf_fmt_s16_dfp() {
    test_femtocontainers_header("bpf_fmt_s16_dfp.c").await;
}

#[tokio::test]
async fn bpf_fmt_u32_dec() {
    test_femtocontainers_header("bpf_fmt_u32_dec.c").await;
}
#[ignore]
#[tokio::test]
async fn inlined_calls() {
    test_femtocontainers_header("inlined_calls.c").await;
}

#[tokio::test]
async fn fletcher_32_checksum() {
    test_femtocontainers_header("fletcher32_checksum_no_strlen.c").await;
}
// For some reason accessing coap packets for femtocontainers VM is broken
// It always fails on the first attempt. I suspect it has something to do with
// how the packetbuffer is initialised. The rbpf implementation works fine.
#[ignore]
#[tokio::test]
async fn gcoap_response_format() {
    test_femtocontainers_header_accessing_coap_pkt("gcoap_response_format.c").await;
}

async fn test_femtocontainers_header(test_program: &str) {
    let env = load_env();
    test_execution(
        test_program,
        BinaryFileLayout::FemtoContainersHeader,
        &env,
    )
    .await;
    test_execution_femtocontainer_vm(
        test_program,
        BinaryFileLayout::FemtoContainersHeader,
        &env,
    )
    .await;
}

/// Tests execution of a given eBPF program which is expected to have access to
/// the incoming network packet that requested the execution of the VM. It
/// then tests whether the response received matches the one specified on the
/// first line of the test file.
async fn test_femtocontainers_header_accessing_coap_pkt(test_program: &str) {
    let env = load_env();
    test_execution_accessing_coap_pkt(
        test_program,
        BinaryFileLayout::FemtoContainersHeader,
        &env,
    )
    .await;

    test_execution_accessing_coap_pkt_femtocontainer_vm(
        test_program,
        BinaryFileLayout::FemtoContainersHeader,
        &env,
    )
    .await;
}

