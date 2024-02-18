use serde::Serialize;
use std::process::Command;

use crate::{args::Action, compile::VmTarget};

#[derive(Serialize)]
struct RequestData {
    pub vm_target: VmTarget,
    pub suit_location: usize,
}

pub async fn handle_execute(args: &crate::args::Action) {
    if let Action::Execute {
        riot_ipv6_addr,
        target,
        suit_storage_slot,
        host_network_interface,
    } = args
    {
        let vm_target = VmTarget::from(target.as_str());

        let request: RequestData = RequestData {
            vm_target,
            suit_location: *suit_storage_slot as usize,
        };

        let url = format!(
            "coap://[{}%{}]/vm/exec/coap-pkt",
            riot_ipv6_addr, host_network_interface
        );

        println!("Sending a request to the url: {}", url);

        let _ = Command::new("aiocoap-client")
            .arg("-m")
            .arg("POST")
            .arg(url.clone())
            .arg("--payload")
            .arg(serde_json::to_string(&request).unwrap())
            .spawn()
            .expect("Failed to send the request.");
    } else {
        panic!("Invalid action args: {:?}", args);
    }
}
