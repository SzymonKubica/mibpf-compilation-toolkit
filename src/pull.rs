use coap::UdpCoAPClient;

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
