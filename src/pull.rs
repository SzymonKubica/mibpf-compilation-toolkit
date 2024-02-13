use std::process::Command;

use crate::args::Action;

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
        let data = format!("{};{}", host_ipv6_addr, suit_manifest);

        let _ = Command::new("aiocoap-client")
            .arg("-m")
            .arg("POST")
            .arg(url.clone())
            .arg("--payload")
            .arg(data.clone())
            .spawn()
            .expect("Failed to send the request.")
            .wait();

    } else {
        panic!("Invalid action args: {:?}", args);
    }
}
