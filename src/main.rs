use std::fs::File;
use std::io::Write;
use std::{path::PathBuf, process::Command};

extern crate rbpf;
use rbpf::helpers;

fn main() {
    let file_name = std::env::args().nth(1).unwrap();

    let base_name = file_name
        .split("/")
        .last()
        .unwrap()
        .split(".")
        .nth(0)
        .expect("You need to provide the .c source file")
        .to_string();

    let obj_file = format!("./out/{}.o", base_name);

    // The first compilation step involves using clang and llvm to compile
    // the eBPF bytecode exactly like it is done in case of the Linux kernel
    // eBPF programs.
    let _ = Command::new("bash")
        .arg("./scripts/compile.sh")
        .arg(file_name)
        .arg(obj_file.clone())
        .spawn()
        .expect("Failed to compile the eBPF bytecode.")
        .wait();

    // The second compilation step patches the bytecode to correct the
    // packet data offsets and replace the instructions so that the packet data
    // is loaded as 8-byte double words.
    let _ = Command::new("bash")
        .arg("./scripts/adjust-bytecode.sh")
        .arg(obj_file.clone())
        .spawn()
        .expect("Failed to patch the eBPF bytecode.")
        .wait();

    // Once the bytecode is patched and the offsets are adjusted correctly
    // we need to strip off the main program section from the object file.
    // This is because only this part is being used by the rbpf VM.

    let path = PathBuf::from(obj_file);
    let file = match elf::File::open_path(&path) {
        Ok(f) => f,
        Err(e) => panic!("Error: {:?}", e),
    };

    // By default we assume the eBPF program is in the ELF section called
    // ".main". If the additional argument is specified, we can override that
    // section name.

    let section_name = if let Some(name) = std::env::args().nth(2) {
        name
    } else {
        ".main".to_string()
    };

    let text_scn = match file.get_section(section_name) {
        Some(s) => s,
        None => panic!("Failed to look up main section"),
    };

    // Extract the program from the elf section.
    let prog = &text_scn.data;

    // The .bin file will contain the bytecode compatible with rbpf.
    let mut f = File::create(format!("./{}.bin", base_name)).unwrap();
    f.write_all(prog.as_slice()).unwrap();

    test_program_run(prog);
}

fn test_program_run(program: &Vec<u8>) {
    let mut packet1 = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x08,
        0x00, // ethertype
        0x45, 0x00, 0x00, 0x3b, // start ip_hdr
        0xa6, 0xab, 0x40, 0x00, 0x40, 0x06, 0x96, 0x0f, 0x7f, 0x00, 0x00, 0x01, 0x7f, 0x00, 0x00,
        0x01,
        // Program matches the next two bytes: 0x9999 returns 0xffffffff, else return 0.
        0x99, 0x99, 0xc6, 0xcc, // start tcp_hdr
        0xd1, 0xe5, 0xc4, 0x9d, 0xd4, 0x30, 0xb5, 0xd2, 0x80, 0x18, 0x01, 0x56, 0xfe, 0x2f, 0x00,
        0x00,
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
}
