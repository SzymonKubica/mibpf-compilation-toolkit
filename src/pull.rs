use std::process::Command;

use serde::{Serialize, Deserialize};

use crate::args::Action;

/// The handler expects to get a request which consists of the IPv6 address of
/// the machine running the CoAP fileserver and the name of the manifest file
/// specifying which binary needs to be pulled.
#[derive(Serialize, Deserialize, Debug)]
struct SuitPullRequest {
    pub ip_addr: String,
    pub manifest: String,
}

pub async fn handle_pull(args: &crate::args::Action) {
    if let Action::Pull {
        riot_ipv6_addr,
        host_ipv6_addr,
        suit_manifest,
        host_network_interface,
    } = args
    {
        let url = format!(
            "coap://[{}%{}]/suit/pull",
            riot_ipv6_addr, host_network_interface
        );
        println!("Sending a request to the url: {}", url);
        let request = SuitPullRequest {
            ip_addr: host_ipv6_addr.clone(),
            manifest: suit_manifest.clone(),
        };

        let req_str = serde_json::to_string(&request).unwrap();
        println!("Checking deserialize: {:?}", serde_json::from_str::<SuitPullRequest>(&req_str));

        let _ = Command::new("aiocoap-client")
            .arg("-m")
            .arg("POST")
            .arg(url.clone())
            .arg("--payload")
            .arg(req_str)
            .spawn()
            .expect("Failed to send the request.")
            .wait();
    } else {
        panic!("Invalid action args: {:?}", args);
    }
}
