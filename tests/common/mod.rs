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


pub async fn test_execution(test_program: &str, layout: BinaryFileLayout) {
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

pub async fn test_execution_accessing_coap_pkt(test_program: &str, layout: BinaryFileLayout) {
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
    assert!(response == expected);
}

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
/// what the expected response from the program executing with access to the CoAP
/// network packet should be.
pub fn extract_expected_response(file_name: &str) -> String {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file_path = format!("{}/{}", TEST_SOURCES_DIR, file_name);
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let first_line = reader.lines().next().unwrap().unwrap();
    // The format of the first line is: // TEST_RESULT: {response}
    let mut first_line_iter = first_line
        .split(" ");

    // We skip the first two tokens: '//' and 'TEST_RESULT' and then collect the
    // rest in case the response contains spaces
    first_line_iter.next();
    first_line_iter.next();

    let response = first_line_iter.collect::<Vec<&str>>().join(" ");
    response
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

/// Sends a request to the server to start executing the program located in
/// the specified storage slot using the functionality of executing eBPF programs
/// that have access to the incoming packet context. The response should be
/// written into the packet buffer by the eBPF program and is returned from
/// this function once we receive it.
pub async fn execute_deployed_program_on_coap(
    suit_storage_slot: usize,
    layout: BinaryFileLayout,
) -> Result<String, String> {
    let available_helpers = (0..23).into_iter().collect::<Vec<u8>>();
    let response = execute(
        ENV.riot_instance_ip,
        TargetVM::Rbpf,
        layout,
        suit_storage_slot,
        ENV.host_net_if,
        ExecutionModel::WithAccessToCoapPacket,
        &available_helpers,
    )
    .await?;

    println!("Response: {}", response);
    // we need to remove the null terminator that we get in the response
    let response = response.trim_matches(char::from(0));
    Ok(response.to_string())
}

pub async fn execute_deployed_program(
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
