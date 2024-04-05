use mibpf_tools::{self, execute};

use internal_representation::{BinaryFileLayout, ExecutionModel, TargetVM};
use mibpf_tools::deploy;
use serde::Deserialize;

pub struct Environment<'a> {
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
pub async fn deploy_test_script(file_name: &str, layout: BinaryFileLayout) -> Result<(), String> {
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

/// Reads the annotation present at the top of test source files that specifies
/// what the expected return value of the program should be.
pub fn extract_expected_return(file_name: &str) -> i32 {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file_path = format!("{}/{}", TEST_SOURCES_DIR, file_name);
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let first_line = reader.lines().next().unwrap().unwrap();
    // The format of the first line is: // TEST_RESULT: 0
    first_line
        .split(" ")
        .nth(2)
        .unwrap()
        .parse::<i32>()
        .unwrap()
}

pub async fn execute_deployed_script(
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
