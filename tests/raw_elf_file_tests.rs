mod common;

use internal_representation::BinaryFileLayout;

use crate::common::{
    deploy_test_script, execute_deployed_program, execute_deployed_program_on_coap,
    extract_expected_response, extract_expected_return,
};

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

#[tokio::test]
async fn printf() {
    test_raw_elf_file("printf.c").await;
}

#[tokio::test]
async fn data_relocations() {
    test_raw_elf_file("data_relocations.c").await;
}

#[tokio::test]
async fn bpf_fetch() {
    test_raw_elf_file("bpf_fetch.c").await;
}

#[tokio::test]
async fn bpf_store() {
    test_raw_elf_file("bpf_store.c").await;
}

#[tokio::test]
async fn bpf_fmt_s16_dfp() {
    test_raw_elf_file("bpf_fmt_s16_dfp.c").await;
}

#[tokio::test]
async fn bpf_fmt_u32_dec() {
    test_raw_elf_file("bpf_fmt_u32_dec.c").await;
}

#[tokio::test]
async fn pc_relative_calls() {
    test_raw_elf_file("pc_relative_calls.c").await;
}


#[tokio::test]
async fn gcoap_response_format() {
    test_raw_elf_file_on_coap("gcoap_response_format.c").await;
}



/// Runs a test which deploys an eBPF script which is prepared to be compatible
/// with [`BinaryFileLayout::RawObjectFile`], the tested implementation on the
/// microcontroller resolves relocations once the program is loaded into memory,
/// then it patches the bytecode and executes the final resulting binary. The
/// return value of the program is returned in the CoAP response that is sent
/// following the request that was sent by this testsuite to the server running
/// on the target device.
///
/// It is important to note that those tests should serve as end-to-end sanity
/// checks rather than a full proof of correctness of the system. Since we can
/// only check for successful execution of the requests we make and then compare
/// the return value of the program to the initial expectation, we cannot guarantee
/// that all parts of the execution of the program were successful. For instance,
/// when testing programs that rely on the bpf_printf helper for logging, the
/// shell output of the tested riot instance should be examined to see if the
/// printed logs are what we would expect them to be.
///
/// Another limitation is that this testsuite was built with the native RIOT
/// instance in mind, which runs as a simulation on the host machine. Because
/// of this, they aren't able to test whether the GPIO-related helpers work
/// as expected.
///
/// Furthermore, because of the design of this testsuite, we can only test programs
/// that terminate quickly enough so that the microcontroller can send the CoAP
/// response with the return value of the program within the request timeout.
async fn test_raw_elf_file(test_program: &str) {
    let layout = BinaryFileLayout::RawObjectFile;

    // We first deploy the program on the tested microcontroller
    let result = deploy_test_script(test_program, layout).await;
    assert!(result.is_ok());

    // Then we request execution and check that the return value is what we
    // expected
    let execution_result = execute_deployed_program(0, layout).await;
    if let Err(string) = &execution_result {
        println!("{}", string);
    }
    assert!(execution_result.is_ok());
    let return_value = execution_result.unwrap();

    let expected_return = extract_expected_return(test_program);
    assert!(return_value == expected_return);
}

/// Tests execution of a given eBPF program which is expected to have access to
/// the incoming network packet that requested the execution of the VM. It
/// then tests whether the response received matches the one specified on the
/// first line of the test file.
async fn test_raw_elf_file_on_coap(test_program: &str) {
    let layout = BinaryFileLayout::RawObjectFile;

    // We first deploy the program on the tested microcontroller
    let result = deploy_test_script(test_program, layout).await;
    assert!(result.is_ok());

    // Then we request execution and check that the return value is what we
    // expected
    let execution_result = execute_deployed_program_on_coap(0, layout).await;
    if let Err(string) = &execution_result {
        println!("{}", string);
    }
    assert!(execution_result.is_ok());
    let response = execution_result.unwrap();

    let expected = extract_expected_response(test_program);
    assert!(response== expected);
}
