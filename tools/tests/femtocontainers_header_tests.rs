mod common;

use common::{
    test_execution, test_execution_accessing_coap_pkt,
    test_execution_accessing_coap_pkt_specifying_helpers, test_execution_specifying_helpers,
};
use mibpf_tools::load_env;

use mibpf_common::{BinaryFileLayout, HelperFunctionID};

// This module contains end-to-end integration tests of the compile-upload-
// execute workflow of the eBPF programs on microcontrollers. It is recommended
// that the tests are run using a native RIOT instance running on the host
// desktop machine.
//
// The tests are set up in a way that each test file contains the expected return
// value on the first line in the source file. This testsuite extracts that information
// and compares it to the actual output returned in the response from the RIOT
// instance running the mibpf server.
//
// TODO: write up setup instructions
// TODO: allow for specifying environment externally
// TODO: update this doc to make it specific to the tested binary file layout.

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
async fn bpf_strlen() {
    test_femtocontainers_header("bpf_strlen.c").await;
}

#[tokio::test]
async fn bpf_fmt_s16_dfp() {
    test_femtocontainers_header("bpf_fmt_s16_dfp.c").await;
}

#[tokio::test]
async fn bpf_fmt_u32_dec() {
    test_femtocontainers_header("bpf_fmt_u32_dec.c").await;
}

#[tokio::test]
async fn pc_relative_calls() {
    test_femtocontainers_header("pc_relative_calls.c").await;
}

#[tokio::test]
async fn inlined_calls() {
    test_femtocontainers_header("inlined_calls.c").await;
}

#[tokio::test]
async fn fletcher_32_checksum() {
    test_femtocontainers_header("fletcher32_checksum.c").await;
}

#[tokio::test]
async fn gcoap_response_format() {
    test_femtocontainers_header_accessing_coap_pkt("gcoap_response_format.c").await;
}

#[tokio::test]
async fn printf_helpers() {
    test_femtocontainers_header_with_helpers(
        "printf.c",
        vec![HelperFunctionID::BPF_PRINTF_IDX.into()],
    )
    .await;
}

#[tokio::test]
async fn bpf_fetch_helpers() {
    test_femtocontainers_header_with_helpers(
        "bpf_fetch.c",
        vec![
            HelperFunctionID::BPF_PRINTF_IDX.into(),
            HelperFunctionID::BPF_FETCH_GLOBAL_IDX.into(),
        ],
    )
    .await;
}

#[tokio::test]
async fn bpf_store_helpers() {
    test_femtocontainers_header_with_helpers(
        "bpf_store.c",
        vec![
            HelperFunctionID::BPF_PRINTF_IDX.into(),
            HelperFunctionID::BPF_STORE_GLOBAL_IDX.into(),
            HelperFunctionID::BPF_FETCH_GLOBAL_IDX.into(),
        ],
    )
    .await;
}

#[tokio::test]
async fn bpf_strlen_helpers() {
    test_femtocontainers_header_with_helpers(
        "bpf_strlen.c",
        vec![
            HelperFunctionID::BPF_PRINTF_IDX.into(),
            HelperFunctionID::BPF_STRLEN_IDX.into(),
        ],
    )
    .await;
}

#[tokio::test]
async fn bpf_fmt_s16_dfp_helpers() {
    test_femtocontainers_header_with_helpers(
        "bpf_fmt_s16_dfp.c",
        vec![
            HelperFunctionID::BPF_PRINTF_IDX.into(),
            HelperFunctionID::BPF_FMT_S16_DFP_IDX.into(),
        ],
    )
    .await;
}

#[tokio::test]
async fn bpf_fmt_u32_dec_helpers() {
    test_femtocontainers_header_with_helpers(
        "bpf_fmt_u32_dec.c",
        vec![
            HelperFunctionID::BPF_PRINTF_IDX.into(),
            HelperFunctionID::BPF_FMT_U32_DEC_IDX.into(),
        ],
    )
    .await;
}

#[tokio::test]
async fn pc_relative_calls_helpers() {
    test_femtocontainers_header_with_helpers(
        "pc_relative_calls.c",
        vec![HelperFunctionID::BPF_PRINTF_IDX.into()],
    )
    .await;
}

#[tokio::test]
async fn inlined_calls_helpers() {
    test_femtocontainers_header_with_helpers(
        "inlined_calls.c",
        vec![HelperFunctionID::BPF_PRINTF_IDX.into()],
    )
    .await;
}

#[tokio::test]
async fn fletcher_32_checksum_helpers() {
    test_femtocontainers_header_with_helpers(
        "fletcher32_checksum.c",
        vec![
            HelperFunctionID::BPF_PRINTF_IDX.into(),
            HelperFunctionID::BPF_STRLEN_IDX.into(),
        ],
    )
    .await;
}

#[tokio::test]
async fn gcoap_response_format_helpers() {
    test_femtocontainers_header_accessing_coap_pkt_with_helpers(
        "gcoap_response_format.c",
        vec![
            HelperFunctionID::BPF_PRINTF_IDX.into(),
            HelperFunctionID::BPF_COAP_ADD_FORMAT_IDX.into(),
            HelperFunctionID::BPF_COAP_OPT_FINISH_IDX.into(),
            HelperFunctionID::BPF_MEMCPY_IDX.into(),
            HelperFunctionID::BPF_GCOAP_RESP_INIT_IDX.into(),
            HelperFunctionID::BPF_FMT_S16_DFP_IDX.into(),
        ],
    )
    .await;
}


async fn test_femtocontainers_header(test_program: &str) {
    let env = load_env();
    test_execution(
        test_program,
        BinaryFileLayout::FemtoContainersHeader,
        &env,
    )
    .await;
}

// Similar to `test_femtocontainers_header` but allows for restricting access
// to helper functions.
async fn test_femtocontainers_header_with_helpers(
    test_program: &str,
    allowed_helpers: Vec<u8>,
) {
    let env = load_env();
    test_execution_specifying_helpers(
        test_program,
        BinaryFileLayout::FemtoContainersHeader,
        &env,
        allowed_helpers,
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
}

async fn test_femtocontainers_header_accessing_coap_pkt_with_helpers(
    test_program: &str,
    allowed_helpers: Vec<u8>,
) {
    let env = load_env();
    test_execution_accessing_coap_pkt_specifying_helpers(
        test_program,
        BinaryFileLayout::FemtoContainersHeader,
        &env,
        allowed_helpers,
    )
    .await;
}
