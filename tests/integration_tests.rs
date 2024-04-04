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
    deploy(
        &file_path,
        TEST_SOURCES_DIR,
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

#[tokio::test]
async fn test_basic() {
    let layout = BinaryFileLayout::RawObjectFile;
    let result = deploy_test_script("printf.c", layout).await;
    assert!(result.is_ok());
    let execution_result = execute_deployed_script(0, layout).await;
    if let Err(string) = &execution_result {
        println!("{}", string);
    }
    assert!(execution_result.is_ok());
    let return_value = execution_result.unwrap();
    assert!(return_value == 0);
}
