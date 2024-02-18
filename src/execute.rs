use std::process::Command;
use serde::{Deserialize, Serialize};

use crate::{args::Action, compile::VmTarget};

#[derive(Serialize)]
enum VmType {
    Rbpf,
    FemtoContainer,
}

#[derive(Serialize)]
struct RequestData {
    pub vm_type: VmType,
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

        // Todo: merge into one endpoint.
        let endpoint = match vm_target {
            VmTarget::FemtoContainers => "rbpf",
            VmTarget::RBPF => "rbpf",
        };

        let request: RequestData = RequestData {
            vm_type: match vm_target {
            VmTarget::FemtoContainers => VmType::FemtoContainer,
            VmTarget::RBPF => VmType::Rbpf,
            },
            suit_location: *suit_storage_slot as usize,
        };

        let url = format!("coap://[{}%{}]/{}/exec", riot_ipv6_addr, host_network_interface, endpoint);

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
