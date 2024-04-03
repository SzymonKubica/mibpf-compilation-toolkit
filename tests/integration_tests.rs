use mibpf_tools;

use mibpf_tools::Action;

// This module contains end-to-end integration tests of the compile-upload-
// execute workflow of the eBPF programs on microcontrollers. It is recommended
// that the tests are run using a native RIOT instance running on the host
// desktop machine.
//
// TODO: write up setup instructions

#[tokio::test]
async fn test_basic() {
    let result = mibpf_tools::handle_deploy(&Action::Deploy {
        bpf_source_file: "tests/test-sources/printf.c".to_string(),
        out_dir: "tests/test-sources".to_string(),
        binary_layout: "RawObjectFile".to_string(),
        host_network_interface: "tapbr0".to_string(),
        riot_network_interface: "6".to_string(),
        board_name: "native".to_string(),
        coaproot_dir: "../coaproot".to_string(),
        suit_storage_slot: 0,
        riot_ipv6_addr: "fe80::a0d9:ebff:fed5:986b".to_string(),
        host_ipv6_addr: "fe80::cc9a:73ff:fe4a:47f6".to_string(),
    }).await;
}
