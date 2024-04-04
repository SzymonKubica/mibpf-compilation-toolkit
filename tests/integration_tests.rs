use std::thread;

use mibpf_tools::{self, execute};

use internal_representation::{BinaryFileLayout, ExecutionModel, TargetVM};
use mibpf_tools::deploy;
use serde::Deserialize;
// This module contains end-to-end integration tests of the compile-upload-
// execute workflow of the eBPF programs on microcontrollers. It is recommended
// that the tests are run using a native RIOT instance running on the host
// desktop machine.
//
// TODO: write up setup instructions
// TODO: allow for specifying environment externally
struct Environment<'a> {
    pub mibpf_root_dir: &'a str,
    pub coap_root_dir: &'a str,
    pub riot_instance_net_if: &'a str,
    pub riot_instance_ip: &'a str,
    pub host_net_if: &'a str,
    pub host_ip: &'a str,
    pub board_name: &'a str,
}

const ENV: Environment = Environment {
    mibpf_root_dir: "..",
    coap_root_dir: "../coaproot",
    riot_instance_net_if: "6",
    riot_instance_ip: "fe80::a0d9:ebff:fed5:986b",
    host_net_if: "tapbr0",
    host_ip: "fe80::cc9a:73ff:fe4a:47f6",
    board_name: "native",
};

const TEST_SOURCES_DIR: &'static str = "tests/test-sources";

/// Test utility funciton used for sending the eBPF scripts to the device given
/// the environment configuration.
async fn deploy_test_script(file_name: &str, layout: BinaryFileLayout) -> Result<(), String> {
    let file_path = format!("{}/{}", TEST_SOURCES_DIR, file_name);
    let out_dir = format!("{}/out", TEST_SOURCES_DIR);
    deploy(
        &file_path,
        &out_dir,
        layout,
        ENV.coap_root_dir,
        0,
        ENV.riot_instance_net_if,
        ENV.riot_instance_ip,
        ENV.host_net_if,
        ENV.host_ip,
        ENV.board_name,
        Some(ENV.mibpf_root_dir),
    )
    .await
}

async fn execute_deployed_script(
    suit_storage_slot: usize,
    layout: BinaryFileLayout,
) -> Result<i32, String> {
    let available_helpers = (0..23).into_iter().collect::<Vec<u8>>();
    let response = execute(
        ENV.riot_instance_ip,
        TargetVM::Rbpf,
        layout,
        suit_storage_slot,
        ENV.host_net_if,
        ExecutionModel::ShortLived,
        &available_helpers,
    )
    .await?;

    // Short lived executions always return responses of this form:
    // {"execution_time": 10, "result": 0}
    #[derive(Deserialize)]
    struct Response {
        // Execution time in milliseconds
        execution_time: u32,
        // Return value of the program
        result: i32,
    }

    println!("Response: {}", response);
    let response = serde_json::from_str::<Response>(&response)
        .map_err(|e| format!("Failed to parse the json response: {}", e))?;

    Ok(response.result)
}

async fn test_raw_elf_file(test_program: &str, expected_return: i32) {
    let layout = BinaryFileLayout::RawObjectFile;

    // We first deploy the program on the tested microcontroller
    let result = deploy_test_script(test_program, layout).await;
    assert!(result.is_ok());

    // Then we request execution and check that the return value is what we
    // expected
    let execution_result = execute_deployed_script(0, layout).await;
    if let Err(string) = &execution_result {
        println!("{}", string);
    }
    assert!(execution_result.is_ok());
    let return_value = execution_result.unwrap();
    assert!(return_value == expected_return);
}

#[tokio::test]
async fn printf() {
    test_raw_elf_file("printf.c", 0).await;
}

#[tokio::test]
async fn data_relocations() {
    test_raw_elf_file("data_relocations.c", 123).await;
}

#[tokio::test]
async fn bpf_fetch() {
    test_raw_elf_file("bpf_fetch.c", 0).await;
}

#[tokio::test]
async fn bpf_store() {
    test_raw_elf_file("bpf_store.c", 1234).await;
}
