use rbpf::helpers;
use serde::Serialize;
use std::{path::PathBuf, process::Command};

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
        execute_on_coap,
    } = args
    {
        let vm_target = VmTarget::from(target.as_str());

        let request: RequestData = RequestData {
            vm_target,
            suit_location: *suit_storage_slot as usize,
        };

        // TODO:make it clean
        if !*execute_on_coap {
            let url = format!(
                "coap://[{}%{}]/vm/spawn",
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
        }

        let endpoint_path = if *execute_on_coap { "/coap-pkt" } else { "" };

        let url = format!(
            "coap://[{}%{}]/vm/exec{}",
            riot_ipv6_addr, host_network_interface, endpoint_path
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

/// Allows for testing generic programs that are loaded into the rBPF VM.
/// It assumes that the program expects to receive a packet with the usual header
/// and ether type fields as well as the payload. It then tries to execute the
/// VM with the loaded program accessing the packet.
pub fn handle_emulate(args: &crate::args::Action) {
    if let Action::EmulateExecution {
        target,
        elf_section_name,
        binary_file,
    } = args
    {
        let path = PathBuf::from(binary_file.clone().unwrap());
        let file = match elf::File::open_path(&path) {
            Ok(f) => f,
            Err(e) => panic!("Error: {:?}", e),
        };

        let text_scn = match file.get_section(elf_section_name) {
            Some(s) => s,
            None => panic!("Failed to look up elf section"),
        };

        // Extract the program from the elf section.
        let program = &text_scn.data;
        let packet1 = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x08,
            0x00, // ethertype
            0x45, 0x00, 0x00, 0x3b, // start ip_hdr
            0xa6, 0xab, 0x40, 0x00, 0x40, 0x06, 0x96, 0x0f, 0x7f, 0x00, 0x00, 0x01, 0x7f, 0x00,
            0x00, 0x01,
            // Program matches the next two bytes: 0x9999 returns 0xffffffff, else return 0.
            0x99, 0x99, 0xc6, 0xcc, // start tcp_hdr
            0xd1, 0xe5, 0xc4, 0x9d, 0xd4, 0x30, 0xb5, 0xd2, 0x80, 0x18, 0x01, 0x56, 0xfe, 0x2f,
            0x00, 0x00,
            // Payload starts here
        ];

        let mut packet_with_payload = packet1.to_vec();

        // This checksum was taken from an example in RIOT.
        let checksum_message = "abcdef\
            AD3Awn4kb6FtcsyE0RU25U7f55Yncn3LP3oEx9Gl4qr7iDW7I8L6Pbw9jNnh0sE4DmCKuc\
        d1J8I34vn31W924y5GMS74vUrZQc08805aj4Tf66HgL1cO94os10V2s2GDQ825yNh9Yuq3\
        QHcA60xl31rdA7WskVtCXI7ruH1A4qaR6Uk454hm401lLmv2cGWt5KTJmr93d3JsGaRRPs\
        4HqYi4mFGowo8fWv48IcA3N89Z99nf0A0H2R6P0uI4Tir682Of3Rk78DUB2dIGQRRpdqVT\
        tLhgfET2gUGU65V3edSwADMqRttI9JPVz8JS37g5QZj4Ax56rU1u0m0K8YUs57UYG5645n\
        byNy4yqxu7";

        let message_bytes = checksum_message.as_bytes();

        // Write message bytes into the packet
        for i in 0..message_bytes.len() {
            print!("{:02x}, ", message_bytes[i]);
        }
        println!("Message length: {}", message_bytes.len());

        packet_with_payload.push(message_bytes.len() as u8);
        packet_with_payload.append(&mut message_bytes.to_vec());

        let mut vm = rbpf::EbpfVmFixedMbuff::new(Some(&program[..]), 0x40, 0x50).unwrap();

        // We register a helper function, that can be called by the program, into
        // the VM.
        vm.register_helper(helpers::BPF_TRACE_PRINTK_IDX, helpers::bpf_trace_printf)
            .unwrap();

        let res = vm.execute_program(&mut packet_with_payload).unwrap();
        println!("Program returned: {:?} ({:#x})", res, res)
    } else {
        panic!("Invalid action args: {:?}", args);
    }
}
