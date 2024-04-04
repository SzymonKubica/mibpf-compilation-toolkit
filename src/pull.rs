use std::process::Command;

use log::debug;

use crate::internal_representation::SuitPullRequest;

pub async fn pull(
    riot_ipv6_addr: &str,
    host_ipv6_addr: &str,
    suit_manifest: &str,
    host_network_interface: &str,
    riot_network_interface: &str,
) -> Result<(), String> {
    let url = format!(
        "coap://[{}%{}]/suit/pull",
        riot_ipv6_addr, host_network_interface
    );
    debug!("Sending a request to the url: {}", url);

    let request = SuitPullRequest {
        ip_addr: host_ipv6_addr.to_string(),
        manifest: suit_manifest.to_string(),
        // We need to tell the microcontroller which network interface (usually 5 or
        // 6) needs to be used to access the CoAP fileserver on the remote host.
        // the reason for this is that this interface changes based on the target
        // architecture (stm32/native) and so it can't be hard-coded.
        riot_network_interface: riot_network_interface.to_string(),
    };

    let req_str = serde_json::to_string(&request).unwrap();

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

    Ok(())
}
