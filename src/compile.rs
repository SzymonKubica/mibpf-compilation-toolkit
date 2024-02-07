use rbpf::helpers;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{fmt, fs};
use std::{path::PathBuf, process::Command};

use crate::args::Action;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmTarget {
    FemtoContainers,
    RBPF,
}

impl From<String> for VmTarget {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Femto-Containers" => VmTarget::FemtoContainers,
            "rBPF" => VmTarget::RBPF,
            _ => panic!("Invalid vm target: {}", s),
        }
    }
}

impl fmt::Display for VmTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn handle_compile(args: &Action) {
    if let Action::Compile {
        bpf_source_file,
        target,
        binary_file,
        out_dir,
        elf_section_name,
        test_execution,
    } = args
    {
        let vm_target = VmTarget::from(target.clone());
        match vm_target {
            VmTarget::FemtoContainers => compile_fc(bpf_source_file, out_dir, binary_file),
            VmTarget::RBPF => compile_rbpf(
                bpf_source_file,
                binary_file,
                out_dir,
                elf_section_name,
                *test_execution,
            ),
        }
    } else {
        panic!("Invalid action args: {:?}", args);
    }
}

fn compile_fc(bpf_source_file: &str, out_dir: &str, binary_file: &Option<String>) {
    let message = "Compiling for Femto-Containers requires header files that \
                   are included in RIOT. Because of this, the compilation \
                   process needs to use the Makefile setup used by RIOT. \
                   You need to ensure that the file {file-name} \
                   you are trying to compile is located inside of a directory \
                   which contains a Makefile that points to RIOT base directory. \
                   See bpf/femto-container directory for an example";
    let formatted_message = message.replace("{file-name}", bpf_source_file);
    println!("[WARNING]\n{}", formatted_message);

    let source_path = PathBuf::from(bpf_source_file);
    let source_directory = source_path.parent().unwrap();

    let _ = Command::new("make")
        .arg("-C")
        .arg(source_directory.to_str().unwrap())
        .arg("clean")
        .arg("all")
        .spawn()
        .expect("Failed to compile the eBPF bytecode.")
        .wait();

    // Make sure the out directory exists
    if !PathBuf::from(out_dir).exists() {
        std::fs::create_dir(out_dir).expect("Failed to create the object file directory.");
    }

    // Copy all of the .o files into the out directory
    // We need to ensure that the out directory exists
    let read_dir = fs::read_dir(source_directory);
    for entry in read_dir.unwrap() {
        let path = &entry.unwrap().path();
        if let Some("o") = path.extension().and_then(OsStr::to_str) {
            let _ = Command::new("mv")
                .arg(path.to_str().unwrap())
                .arg(out_dir)
                .spawn()
                .expect("Failed to copy the binary file.")
                .wait();
        }
    }

    if let Some(file_name) = binary_file {
        let base_name = bpf_source_file
            .split("/")
            .last()
            .unwrap()
            .split(".")
            .nth(0)
            .expect("You need to provide the .c source file")
            .to_string();

        let _ = Command::new("mv")
            .arg(format!(
                "{}/{}.bin",
                source_directory.to_str().unwrap(),
                base_name
            ))
            .arg(file_name)
            .spawn()
            .expect("Failed to copy the binary file.")
            .wait();
    }
}

fn compile_rbpf(
    bpf_source_file: &str,
    binary_file: &Option<String>,
    out_dir: &str,
    elf_section_name: &str,
    test_execution: bool,
) {
    let file_name = bpf_source_file;

    let base_name = file_name
        .split("/")
        .last()
        .unwrap()
        .split(".")
        .nth(0)
        .expect("You need to provide the .c source file")
        .to_string();

    let obj_file = format!("{}/{}.o", out_dir, base_name);

    // We need to ensure that the out directory exists
    if !PathBuf::from(out_dir).exists() {
        std::fs::create_dir(out_dir).expect("Failed to create the object file directory.");
    }

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

    let text_scn = match file.get_section(elf_section_name) {
        Some(s) => s,
        None => panic!("Failed to look up elf section"),
    };

    // Extract the program from the elf section.
    let prog = &text_scn.data;

    let output_file_name = if let Some(binary_file) = binary_file {
        binary_file.clone()
    } else {
        let binary_file = format!("./{}.bin", base_name);
        binary_file
    };

    // The .bin file will contain the bytecode compatible with rbpf.
    let mut f = File::create(output_file_name).unwrap();
    f.write_all(prog.as_slice()).unwrap();

    if test_execution {
        test_program_execution(prog);
    }
}

fn test_program_execution(program: &Vec<u8>) {
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
