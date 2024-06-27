use std::{env, process::Command, collections::HashMap};

use enum_iterator::all;
use micro_bpf_tools::{self, deploy, execute, Environment};

use micro_bpf_common::{
    BinaryFileLayout, ExecutionModel, HelperAccessListSource, HelperAccessVerification,
    HelperFunctionID, TargetVM, VMConfiguration, VMExecutionRequest,
};
use serde::{Deserialize, Serialize};

/// When communicating with target board sometimes it takes longer to get the request processed
/// we need to wait a bit longer to give the device time to respons
const NUCLEO_EXECUTION_REQUEST_TIMEOUT: u64 = 1;

pub async fn test_execution(
    test_program: &str,
    layout: BinaryFileLayout,
    environment: &Environment,
) {
    // By default all helpers are allowed
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    test_execution_specifying_helpers(
        test_program,
        layout,
        TargetVM::Rbpf,
        environment,
        available_helpers,
    )
    .await;
}

pub async fn test_jit_execution(
    test_program: &str,
    layout: BinaryFileLayout,
    environment: &Environment,
) {
    // By default all helpers are allowed
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    test_jit_execution_specifying_helpers(
        test_program,
        layout,
        TargetVM::Rbpf,
        environment,
        available_helpers,
    )
    .await;
}
pub async fn test_execution_femtocontainer_vm(
    test_program: &str,
    layout: BinaryFileLayout,
    environment: &Environment,
) {
    // By default all helpers are allowed
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    test_execution_specifying_helpers(
        test_program,
        layout,
        TargetVM::FemtoContainer,
        environment,
        available_helpers,
    )
    .await;
}

pub async fn test_jit_execution_specifying_helpers(
    test_program: &str,
    layout: BinaryFileLayout,
    target_vm: TargetVM,
    environment: &Environment,
    available_helpers: Vec<u8>,
) {
    // We first deploy the program on the tested microcontroller
    let result = deploy_test_script(test_program, layout, environment, available_helpers).await;
    if let Err(string) = &result {
        println!("{}", string);
    }
    assert!(result.is_ok());

    // When running on embedded targets we need to give them enough time
    // to fetch the firmware
    if environment.board_name != "native" {
        std::thread::sleep(std::time::Duration::from_secs(
            NUCLEO_EXECUTION_REQUEST_TIMEOUT,
        ));
    }

    // Then we request execution and check that the return value is what we
    // expected
    let execution_result = execute_deployed_program(0, layout, target_vm, environment, true).await;
    if let Err(string) = &execution_result {
        println!("{}", string);
    }
    assert!(execution_result.is_ok());
    let return_value = execution_result.unwrap();

    let expected_return = extract_expected_return(test_program);
    assert!(return_value == expected_return);
}

pub async fn test_execution_specifying_helpers(
    test_program: &str,
    layout: BinaryFileLayout,
    target_vm: TargetVM,
    environment: &Environment,
    available_helpers: Vec<u8>,
) {
    // We first deploy the program on the tested microcontroller
    let result = deploy_test_script(test_program, layout, environment, available_helpers).await;
    if let Err(string) = &result {
        println!("{}", string);
    }
    assert!(result.is_ok());

    // When running on embedded targets we need to give them enough time
    // to fetch the firmware
    if environment.board_name != "native" {
        std::thread::sleep(std::time::Duration::from_secs(
            NUCLEO_EXECUTION_REQUEST_TIMEOUT,
        ));
    }

    // Then we request execution and check that the return value is what we
    // expected
    let execution_result = execute_deployed_program(0, layout, target_vm, environment, false).await;
    if let Err(string) = &execution_result {
        println!("{}", string);
    }
    assert!(execution_result.is_ok());
    let return_value = execution_result.unwrap();

    let expected_return = extract_expected_return(test_program);
    assert!(return_value == expected_return);
}

pub async fn benchmark_fletcher_16(
    data_size: usize,
    environment: &Environment,
    layout: BinaryFileLayout,
    jit: bool,
) -> BenchmarkResponse {
    let available_helpers = vec![HelperFunctionID::BPF_STRLEN_IDX as u8];

    let base: u32 = 2;
    let bytes = 80 * base.pow((data_size - 1) as u32);
    let test_source = format!("jit_fletcher16_checksum_{}B_data.c", bytes);

    let result =
        deploy_test_script(&test_source, layout, environment, available_helpers.clone()).await;
    if let Err(string) = &result {
        println!("{}", string);
    }
    assert!(result.is_ok());

    // When running on embedded targets we need to give them enough time
    // to fetch the firmware
    std::thread::sleep(std::time::Duration::from_secs(
        NUCLEO_EXECUTION_REQUEST_TIMEOUT,
    ));

    let target_vm = match layout {
        BinaryFileLayout::FemtoContainersHeader => TargetVM::FemtoContainer,
        _ => TargetVM::Rbpf,
    };

    let response = execute(
        &environment.riot_instance_ip,
        target_vm,
        layout,
        0,
        &environment.host_net_if,
        ExecutionModel::ShortLived,
        HelperAccessVerification::AheadOfTime,
        HelperAccessListSource::ExecuteRequest,
        &available_helpers,
        jit,
        true,
        true,
    )
    .await
    .unwrap();
    let response = serde_json::from_str::<BenchmarkResponse>(&response).unwrap();
    println!("({}, {})", bytes, response.execution_time);
    return response;
}

pub async fn benchmark_fletcher_16_native(
    data_size: usize,
    environment: &Environment,
) -> SimpleResponse {
    // The size of the benchmarked
    let available_helpers = all::<HelperFunctionID>()
        .take(data_size)
        .collect::<Vec<HelperFunctionID>>();

    let request = VMExecutionRequest::new(
        // We make a dummy request to the native jit endpoint
        VMConfiguration::new(
            TargetVM::FemtoContainer,
            0,
            BinaryFileLayout::FemtoContainersHeader,
            HelperAccessVerification::AheadOfTime,
            HelperAccessListSource::ExecuteRequest,
            false,
            false,
        ),
        available_helpers,
    );

    let mut base_url = format!(
        "coap://[{}%{}]",
        environment.riot_instance_ip, environment.host_net_if
    );
    // We need to point to the native benchmark endpoint.
    base_url.push_str("/native/exec");

    let payload = request.encode();

    // We use the aiocoap-client here as opposed to the rust coap library because
    // that one didn't support overriding the network interface in the ipv6 urls
    let Ok(output) = Command::new("aiocoap-client")
        .arg("-m")
        .arg("POST")
        .arg(base_url.clone())
        .arg("--payload")
        .arg(&payload)
        .output()
    else {
        panic!("Failed to execute the command");
    };

    if output.stderr.len() > 0 {
        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| format!("Failed to parse the stderr: {}", e));
        panic!(
            "{}",
            format!("aiocoap-client failed with: {}", stderr.unwrap())
        );
    }

    let response = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse the response: {}", e))
        .unwrap();

    let base: u32 = 2;
    let bytes = 80 * base.pow((data_size - 1) as u32);
    let response = serde_json::from_str::<SimpleResponse>(&response).unwrap();
    println!("({}, {})", bytes, response.execution_time);
    response
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SimpleResponse {
    execution_time: u32,
    result: i32,
}

/// This benchmark aims at showing that as the list of allowed memory regions
/// grows, the original memory access check approach becomes very slow. This is
/// because each time memory is accessed, the list of allowed regions is traversed
/// and as the size of the list grows, so doesn the traversal time. Running this
/// test with the environment variable CACHE_MEM_CHECKS=true improves the
/// performance.
///
/// The experimental setup is the following:
/// We run two example programs:
/// looping.c
/// data_relocations-looping.c
///
/// The first one loops by incrementing a variable on the stack, the
/// second one increments a variable in the .data section. The code of rbpf
/// is set up so that as the list of allowed regions grows, the stack remains
/// as the first item in the list, whereas the .data section shifts further
/// to the back. This is done to exhibit a worst-case scenario where we need
/// to access an item which is present in one of the memory regions located at
/// the end of the list.
///
/// Because of the above setup, running the memory checks without caching should
/// cause the second program to execute much slower as the number of allowed
/// regions grows.
pub async fn benchmark_memory_access_checks(environment: &Environment) -> HashMap<&'static str, HashMap<usize, BenchmarkResponse>> {
    let region_list_sizes = vec![1, 4, 8, 12, 16];
    // The number of memory regions is controlled by the size of available helpers
    // (this is an instrumentation hack, in normal applications the list of
    // allowed regions should be set up according to application requirements).
    let mut stack_memory_access_benches = HashMap::new();
    let mut data_section_memory_access_benches = HashMap::new();
    let result = deploy_test_script_into_slot(
        "looping.c",
        BinaryFileLayout::RawObjectFile,
        environment,
        vec![],
        0,
    )
    .await;

    if let Err(string) = &result {
        println!("{}", string);
    }

    assert!(result.is_ok());

    // When running on embedded targets we need to give them enough time
    // to fetch the firmware
    if environment.board_name != "native" {
        std::thread::sleep(std::time::Duration::from_secs(
            NUCLEO_EXECUTION_REQUEST_TIMEOUT,
        ));
    }
    let result2 = deploy_test_script_into_slot(
        "data_relocations-looping.c",
        BinaryFileLayout::RawObjectFile,
        environment,
        vec![],
        1,
    )
    .await;

    if let Err(string) = &result2 {
        println!("{}", string);
    }

    assert!(result2.is_ok());
    // When running on embedded targets we need to give them enough time
    // to fetch the firmware
    if environment.board_name != "native" {
        std::thread::sleep(std::time::Duration::from_secs(
            NUCLEO_EXECUTION_REQUEST_TIMEOUT,
        ));
    }

    for size in region_list_sizes {
        let available_helpers = all::<HelperFunctionID>()
            .take(size)
            .map(|e| e as u8)
            .collect::<Vec<u8>>();
        if let Err(string) = &result {
            println!("{}", string);
        }
        assert!(result.is_ok());

        let response = execute(
            &environment.riot_instance_ip,
            TargetVM::Rbpf,
            BinaryFileLayout::RawObjectFile,
            0,
            &environment.host_net_if,
            ExecutionModel::ShortLived,
            HelperAccessVerification::AheadOfTime,
            HelperAccessListSource::ExecuteRequest,
            &available_helpers,
            false,
            false,
            true,
        )
        .await
        .unwrap();
        // We need to use the short names in the json as the COAP packet that we
        // can send to the microcontroller is limited in size.

        println!("Response: {}", response);
        let response = serde_json::from_str::<BenchmarkResponse>(&response)
            .map_err(|e| format!("Failed to parse the json response: {}", e))
            .unwrap();

        stack_memory_access_benches.insert(size, response);

        // When running on embedded targets we need to give them enough time
        // to fetch the firmware
        if environment.board_name != "native" {
            std::thread::sleep(std::time::Duration::from_secs(
                NUCLEO_EXECUTION_REQUEST_TIMEOUT,
            ));
        }

        let response = execute(
            &environment.riot_instance_ip,
            TargetVM::Rbpf,
            BinaryFileLayout::RawObjectFile,
            1,
            &environment.host_net_if,
            ExecutionModel::ShortLived,
            HelperAccessVerification::AheadOfTime,
            HelperAccessListSource::ExecuteRequest,
            &available_helpers,
            false,
            false,
            true,
        )
        .await
        .unwrap();
        // We need to use the short names in the json as the COAP packet that we
        // can send to the microcontroller is limited in size.

        println!("Response: {}", response);
        let response = serde_json::from_str::<BenchmarkResponse>(&response)
            .map_err(|e| format!("Failed to parse the json response: {}", e))
            .unwrap();

        data_section_memory_access_benches.insert(size, response);
    }

    let mut result = HashMap::new();
    result.insert("stack_memory_access", stack_memory_access_benches);
    result.insert("data_section_memory_access", data_section_memory_access_benches);
    result
}

pub fn save_results<T: serde::Serialize>(file_name: &str, results: T) {
    let save_results = env::var("SAVE_RESULTS").unwrap_or_else(|_| "False".to_string());
    match save_results.as_str() {
        "False" => return,
        _ => (),
    }
    let _ = std::fs::write(file_name, serde_json::to_string_pretty(&results).unwrap());
}

pub async fn benchmark_execution(
    test_program: &str,
    layout: BinaryFileLayout,
    environment: &Environment,
    target: TargetVM,
) -> BenchmarkResponse {
    // By default all helpers are allowed
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    // We first deploy the program on the tested microcontroller
    let result = deploy_test_script(test_program, layout, environment, available_helpers).await;
    if let Err(string) = &result {
        println!("{}", string);
    }
    assert!(result.is_ok());

    // When running on embedded targets we need to give them enough time
    // to fetch the firmware
    if environment.board_name != "native" {
        std::thread::sleep(std::time::Duration::from_secs(
            NUCLEO_EXECUTION_REQUEST_TIMEOUT,
        ));
    }

    // when executing a different helper encoding is used.

    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    let response = execute(
        &environment.riot_instance_ip,
        target,
        layout,
        0,
        &environment.host_net_if,
        ExecutionModel::ShortLived,
        HelperAccessVerification::AheadOfTime,
        HelperAccessListSource::ExecuteRequest,
        &available_helpers,
        false,
        false,
        true,
    )
    .await
    .unwrap();
    // We need to use the short names in the json as the COAP packet that we
    // can send to the microcontroller is limited in size.

    println!("Response: {}", response);
    let response = serde_json::from_str::<BenchmarkResponse>(&response)
        .map_err(|e| format!("Failed to parse the json response: {}", e))
        .unwrap();

    response
}

#[derive(Deserialize, Debug, Serialize)]
pub struct BenchmarkResponse {
    #[serde(rename(deserialize = "total"))]
    total_time: u32,
    #[serde(rename(deserialize = "load"))]
    load_time: u32,
    #[serde(rename(deserialize = "verif"))]
    verification_time: u32,
    // Execution time in milliseconds
    #[serde(rename(deserialize = "exec"))]
    execution_time: u32,
    // Return value of the program
    #[serde(rename(deserialize = "prog"))]
    program_size: u32,
    result: i32,
}

pub async fn benchmark_jit_execution(
    test_program: &str,
    environment: &Environment,
) -> BenchmarkResponse {
    // By default all helpers are allowed
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    // We first deploy the program on the tested microcontroller
    let layout = BinaryFileLayout::RawObjectFile;
    let result = deploy_test_script(test_program, layout, environment, available_helpers).await;
    if let Err(string) = &result {
        println!("{}", string);
    }
    assert!(result.is_ok());

    // When running on embedded targets we need to give them enough time
    // to fetch the firmware
    if environment.board_name != "native" {
        std::thread::sleep(std::time::Duration::from_secs(
            NUCLEO_EXECUTION_REQUEST_TIMEOUT,
        ));
    }

    // when executing a different helper encoding is used.

    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    let response = execute(
        &environment.riot_instance_ip,
        TargetVM::Rbpf,
        layout,
        0,
        &environment.host_net_if,
        ExecutionModel::ShortLived,
        HelperAccessVerification::AheadOfTime,
        HelperAccessListSource::ExecuteRequest,
        &available_helpers,
        true,
        true,
        true,
    )
    .await
    .unwrap();

    println!("Response: {}", response);
    let response = serde_json::from_str::<BenchmarkResponse>(&response)
        .map_err(|e| format!("Failed to parse the json response: {}", e))
        .unwrap();
    response
}

pub async fn test_execution_accessing_coap_pkt_femtocontainer_vm(
    test_program: &str,
    layout: BinaryFileLayout,
    environment: &Environment,
) {
    // By default all helpers are allowed
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    test_execution_accessing_coap_pkt_specifying_helpers(
        test_program,
        layout,
        TargetVM::FemtoContainer,
        environment,
        available_helpers,
        false,
    )
    .await
}

pub async fn test_execution_accessing_coap_pkt(
    test_program: &str,
    layout: BinaryFileLayout,
    environment: &Environment,
) {
    // By default all helpers are allowed
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    test_execution_accessing_coap_pkt_specifying_helpers(
        test_program,
        layout,
        TargetVM::Rbpf,
        environment,
        available_helpers,
        false,
    )
    .await
}

pub async fn test_execution_accessing_coap_pkt_specifying_helpers(
    test_program: &str,
    layout: BinaryFileLayout,
    target_vm: TargetVM,
    environment: &Environment,
    available_helpers: Vec<u8>,
    jit: bool,
) {
    // We first deploy the program on the tested microcontroller
    let result = deploy_test_script(test_program, layout, environment, available_helpers).await;
    if let Err(string) = &result {
        println!("{}", string);
    }

    // When running on embedded targets we need to give them enough time
    // to fetch the firmware
    if environment.board_name != "native" {
        std::thread::sleep(std::time::Duration::from_secs(
            NUCLEO_EXECUTION_REQUEST_TIMEOUT,
        ));
    }
    assert!(result.is_ok());

    // Then we request execution and check that the return value is what we
    // expected
    let execution_result =
        execute_deployed_program_on_coap(0, layout, target_vm, environment, jit).await;
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
pub async fn deploy_test_script(
    file_name: &str,
    layout: BinaryFileLayout,
    environment: &Environment,
    allowed_helpers: Vec<u8>,
) -> Result<(), String> {
    let file_path = format!("{}/{}", TEST_SOURCES_DIR, file_name);
    let out_dir = format!("{}/out", TEST_SOURCES_DIR);
    deploy(
        &file_path,
        &out_dir,
        TargetVM::Rbpf,
        layout,
        &environment.coap_root_dir,
        0,
        &environment.riot_instance_net_if,
        &environment.riot_instance_ip,
        &environment.host_net_if,
        &environment.host_ip,
        &environment.board_name,
        Some(environment.micro_bpf_root_dir.as_str()),
        allowed_helpers,
        HelperAccessVerification::AheadOfTime,
        HelperAccessListSource::ExecuteRequest,
        true,
    )
    .await
}

pub async fn deploy_test_script_into_slot(
    file_name: &str,
    layout: BinaryFileLayout,
    environment: &Environment,
    allowed_helpers: Vec<u8>,
    suit_slot: usize,
) -> Result<(), String> {
    let file_path = format!("{}/{}", TEST_SOURCES_DIR, file_name);
    let out_dir = format!("{}/out", TEST_SOURCES_DIR);
    deploy(
        &file_path,
        &out_dir,
        TargetVM::Rbpf,
        layout,
        &environment.coap_root_dir,
        suit_slot,
        &environment.riot_instance_net_if,
        &environment.riot_instance_ip,
        &environment.host_net_if,
        &environment.host_ip,
        &environment.board_name,
        Some(environment.micro_bpf_root_dir.as_str()),
        allowed_helpers,
        HelperAccessVerification::AheadOfTime,
        HelperAccessListSource::ExecuteRequest,
        true,
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
    let mut first_line_iter = first_line.split(" ");

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
    target_vm: TargetVM,
    environment: &Environment,
    jit: bool,
) -> Result<String, String> {
    // We allow all helpers
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    let response = execute(
        &environment.riot_instance_ip,
        target_vm,
        layout,
        suit_storage_slot,
        &environment.host_net_if,
        ExecutionModel::WithAccessToCoapPacket,
        HelperAccessVerification::AheadOfTime,
        HelperAccessListSource::ExecuteRequest,
        &available_helpers,
        jit,
        true,
        false,
    )
    .await?;

    println!("Response: {}", response);
    // we need to remove the null terminator that we get in the response
    let response = response.trim_matches(char::from(0));
    Ok(response.to_string())
}

pub async fn execute_deployed_program_specifying_helpers(
    suit_storage_slot: usize,
    layout: BinaryFileLayout,
    target_vm: TargetVM,
    environment: &Environment,
    available_helpers: Vec<u8>,
    jit: bool,
) -> Result<i32, String> {
    let response = execute(
        &environment.riot_instance_ip,
        target_vm,
        layout,
        suit_storage_slot,
        &environment.host_net_if,
        ExecutionModel::ShortLived,
        HelperAccessVerification::AheadOfTime,
        HelperAccessListSource::ExecuteRequest,
        &available_helpers,
        jit,
        true,
        false,
    )
    .await?;

    // Short lived executions always return responses of this form:
    // {"execution_time": 10, "result": 0}
    #[derive(Deserialize)]
    struct Response {
        // Return value of the program
        result: i32,
    }

    println!("Response: {}", response);
    let response = serde_json::from_str::<Response>(&response)
        .map_err(|e| format!("Failed to parse the json response: {}", e))?;

    Ok(response.result)
}

pub async fn execute_deployed_program(
    suit_storage_slot: usize,
    layout: BinaryFileLayout,
    target_vm: TargetVM,
    environment: &Environment,
    jit: bool,
) -> Result<i32, String> {
    let available_helpers = all::<HelperFunctionID>()
        .map(|e| e as u8)
        .collect::<Vec<u8>>();
    execute_deployed_program_specifying_helpers(
        suit_storage_slot,
        layout,
        target_vm,
        environment,
        available_helpers,
        jit,
    )
    .await
}
