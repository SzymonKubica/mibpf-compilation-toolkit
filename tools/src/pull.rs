use std::process::Command;

use log::{debug, error};
use mibpf_common::{
    BinaryFileLayout, HelperAccessListSource, HelperAccessVerification, TargetVM, VMConfiguration,
};

use crate::mibpf_common::SuitPullRequest;

pub async fn pull(
    riot_ipv6_addr: &str,
    host_ipv6_addr: &str,
    suit_manifest: &str,
    host_network_interface: &str,
    riot_network_interface: &str,
    target: TargetVM,
    binary_layout: BinaryFileLayout,
    suit_storage_slot: usize,
    helper_access_verification: HelperAccessVerification,
    helper_access_list_source: HelperAccessListSource,
    helper_indices: &[u8],
    erase: bool,
) -> Result<(), String> {
    let url = format!(
        "coap://[{}%{}]/suit/pull",
        riot_ipv6_addr, host_network_interface
    );
    debug!("Sending a request to the url: {}", url);

    let configuration = VMConfiguration::new(
        target,
        suit_storage_slot,
        binary_layout,
        helper_access_verification,
        helper_access_list_source,
    );

    let config_encoded = configuration.encode();

    let request = SuitPullRequest {
        ip: host_ipv6_addr.to_string(),
        manifest: suit_manifest.to_string(),
        // We need to tell the microcontroller which network interface (usually 5 or
        // 6) needs to be used to access the CoAP fileserver on the remote host.
        // the reason for this is that this interface changes based on the target
        // architecture (stm32/native) and so it can't be hard-coded.
        riot_netif: riot_network_interface.to_string(),
        config: config_encoded,
        helpers: helper_indices
            .iter()
            .map(|i| format!("{:02x}", i))
            .collect::<String>(),
        erase
    };

    let req_str = request.encode();
    debug!("Sending the request payload: {}", req_str);

    let Ok(output) = Command::new("aiocoap-client")
        .arg("-m")
        .arg("POST")
        .arg(url.clone())
        .arg("--payload")
        .arg(&req_str)
        .output()
    else {
        return Err(format!("Failed to send the request payload: {}", req_str));
    };

    debug!(
        "Response from the pull request: \n{}",
        String::from_utf8(output.stdout).unwrap()
    );

    if output.stderr.len() > 0 {
        error!("{}", String::from_utf8(output.stderr).unwrap_or_default());
    }

    Ok(())
}
