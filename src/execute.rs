use std::process::Command;

use crate::{args::Action, compile::VmTarget};

pub async fn handle_execute(args: &crate::args::Action) {
    if let Action::Execute {
        riot_ipv6_addr,
        target,
        suit_storage_slot,
        host_network_interface,
    } = args
    {
        let vm_target = VmTarget::from(target.as_str());

        let endpoint = match vm_target {
            VmTarget::FemtoContainers => "femto-container",
            VmTarget::RBPF => "rbpf",
        };

        let url = format!("coap://[{}%{}]/{}/exec", riot_ipv6_addr, host_network_interface, endpoint);

        println!("Sending a request to the url: {}", url);

        let _ = Command::new("aiocoap-client")
            .arg("-m")
            .arg("POST")
            .arg(url.clone())
            .arg("--payload")
            .arg(suit_storage_slot.to_string())
            .spawn()
            .expect("Failed to send the request.");


    } else {
        panic!("Invalid action args: {:?}", args);
    }
}
