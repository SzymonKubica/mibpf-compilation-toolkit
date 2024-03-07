use std::process::Command;

use log::debug;

use crate::{args::Action, internal_representation::SuitPullRequest};

pub async fn handle_pull(args: &crate::args::Action) -> Result<(), String> {
    let Action::Pull {
        riot_ipv6_addr,
        host_ipv6_addr,
        suit_manifest,
        host_network_interface,
    } = args
    else {
        return Err(format!("Invalid action args: {:?}", args));
    };

    let url = format!(
        "coap://[{}%{}]/suit/pull",
        riot_ipv6_addr, host_network_interface
    );
    debug!("Sending a request to the url: {}", url);

    let request = SuitPullRequest {
        ip_addr: host_ipv6_addr.clone(),
        manifest: suit_manifest.clone(),
    };

    let req_str = serde_json::to_string(&request).unwrap();

    let Ok(_) = Command::new("aiocoap-client")
        .arg("-m")
        .arg("POST")
        .arg(url.clone())
        .arg("--payload")
        .arg(&req_str)
        .spawn()
        .expect("Failed to send the request.")
        .wait()
    else {
        return Err(format!("Failed to send the request payload: {}", req_str));
    };

    Ok(())
}
