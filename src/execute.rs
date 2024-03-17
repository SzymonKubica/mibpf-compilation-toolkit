use std::process::Command;

use log::debug;

use crate::{
    args::Action,
    internal_representation::{BinaryFileLayout, ExecuteRequest, VmTarget},
};

pub async fn handle_execute(args: &crate::args::Action) -> Result<(), String> {
    let Action::Execute {
        riot_ipv6_addr: riot_ipv6,
        target,
        binary_layout,
        suit_storage_slot,
        host_network_interface: host_net_if,
        execute_on_coap,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };
    let vm_target = VmTarget::from(target.as_str());

    let binary_layout = BinaryFileLayout::from(binary_layout.as_str());

    let request: ExecuteRequest = ExecuteRequest {
        vm_target,
        binary_layout,
        suit_slot: *suit_storage_slot as usize,
        allowed_helpers: 0,
    };

    let url = if !*execute_on_coap {
        format!("coap://[{}%{}]/vm/spawn", riot_ipv6, host_net_if)
    } else {
        format!("coap://[{}%{}]/vm/exec/coap-pkt", riot_ipv6, host_net_if,)
    };

    debug!("Sending a request to the url: {}", url);

    let payload = serde_json::to_string(&request).unwrap();

    let Ok(_) = Command::new("aiocoap-client")
        .arg("-m")
        .arg("POST")
        .arg(url.clone())
        .arg("--payload")
        .arg(&payload)
        .spawn()
        .expect("Failed to send the request.")
        .wait()
    else {
        return Err(format!("Failed to send request payload: {}", payload));
    };

    Ok(())
}
