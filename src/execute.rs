use coap::UdpCoAPClient;

use crate::{args::Action, compile::VmTarget};

pub async fn handle_execute(args: &crate::args::Action) {
    if let Action::Execute {
        riot_ipv6_addr,
        target,
        suit_storage_slot,
    } = args
    {

        let vm_target = VmTarget::from(target.as_str());

        let endpoint = match vm_target {
            VmTarget::FemtoContainers => "femto-containers",
            VmTarget::RBPF => "rbpf",
        };

        let url = format!("coap://[{}]/{}/pull", riot_ipv6_addr, endpoint);
        println!("Sending a request to the url: {}", url);

        let data = format!("{}", suit_storage_slot);

        let response = UdpCoAPClient::post(url.as_ref(), data.as_bytes().to_vec())
            .await
            .unwrap();

        println!(
            "Server reply: {}",
            String::from_utf8(response.message.payload).unwrap()
        );
    } else {
        panic!("Invalid action args: {:?}", args);
    }
}
