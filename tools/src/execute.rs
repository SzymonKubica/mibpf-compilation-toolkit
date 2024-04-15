use std::process::Command;

use enum_iterator::all;
use log::debug;
use mibpf_common::{
    ExecutionModel, HelperAccessListSource, HelperAccessVerification, HelperFunctionID,
};

use crate::mibpf_common::{BinaryFileLayout, TargetVM, VMConfiguration, VMExecutionRequest};

pub async fn execute(
    riot_ipv6_addr: &str,
    target: TargetVM,
    binary_layout: BinaryFileLayout,
    suit_storage_slot: usize,
    host_network_interface: &str,
    execution_model: ExecutionModel,
    helper_access_verification: HelperAccessVerification,
    helper_access_list_source: HelperAccessListSource,
    helper_indices: &[u8],
) -> Result<String, String> {
    // If the user doesn't specify any allowed helper indices, we allow all of them
    // by default.
    let helper_indices = if helper_indices.len() == 0 {
        all::<HelperFunctionID>().collect::<Vec<HelperFunctionID>>()
    } else {
        helper_indices
            .to_vec()
            .into_iter()
            .filter_map(|i| num::FromPrimitive::from_u8(i))
            .collect::<Vec<HelperFunctionID>>()
    };

    let request = VMExecutionRequest::new(
        VMConfiguration::new(
            target,
            suit_storage_slot,
            binary_layout,
            helper_access_verification,
            helper_access_list_source,
        ),
        helper_indices,
    );

    println!("Helper encoding: {:?}", request.allowed_helpers);

    let url = match execution_model {
        ExecutionModel::ShortLived => format!(
            "coap://[{}%{}]/vm/exec",
            riot_ipv6_addr, host_network_interface
        ),
        ExecutionModel::WithAccessToCoapPacket => {
            format!(
                "coap://[{}%{}]/vm/exec/coap-pkt",
                riot_ipv6_addr, host_network_interface
            )
        }
        ExecutionModel::LongRunning => format!(
            "coap://[{}%{}]/vm/spawn",
            riot_ipv6_addr, host_network_interface
        ),
        ExecutionModel::Benchmark => format!(
            "coap://[{}%{}]/vm/bench",
            riot_ipv6_addr, host_network_interface
        ),
    };

    debug!("Sending a request to the url: {}", url);

    let payload = request.encode();

    // We use the aiocoap-client here as opposed to the rust coap library because
    // that one didn't support overriding the network interface in the ipv6 urls
    let Ok(output) = Command::new("aiocoap-client")
        .arg("-m")
        .arg("POST")
        .arg(url.clone())
        .arg("--payload")
        .arg(&payload)
        .output()
    else {
        return Err(format!("Failed to send request payload: {}", payload));
    };

    if output.stderr.len() > 0 {
        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| format!("Failed to parse the stderr: {}", e))?;
        Err(format!("aiocoap-client failed with: {}", stderr))?
    }

    let response = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse the response: {}", e))?;

    Ok(response)
}
