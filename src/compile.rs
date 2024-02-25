use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::{fmt, fs};
use std::{path::PathBuf, process::Command};
use serde::Serialize;

use crate::args::Action;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum VmTarget {
    FemtoContainer,
    Rbpf,
}

impl From<&str> for VmTarget {
    fn from(s: &str) -> Self {
        match s {
            "FemtoContainer" => VmTarget::FemtoContainer,
            "rBPF" => VmTarget::Rbpf,
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
    } = args
    {
        let vm_target = VmTarget::from(target.as_str());
        match vm_target {
            VmTarget::FemtoContainer => compile_fc(bpf_source_file, out_dir, binary_file),
            VmTarget::Rbpf => compile_rbpf(
                bpf_source_file,
                binary_file,
                out_dir,
                elf_section_name,
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

    // Move all of the .o and .bin files into the out directory
    // Make sure the out directory exists
    if !PathBuf::from(out_dir).exists() {
        std::fs::create_dir(out_dir).expect("Failed to create the object file directory.");
    }

    let read_dir = fs::read_dir(source_directory);
    for entry in read_dir.unwrap() {
        let path = &entry.unwrap().path();
        let extension = path.extension().and_then(OsStr::to_str);
        if Some("o") == extension || Some("bin") == extension {
            let _ = Command::new("mv")
                .arg(path.to_str().unwrap())
                .arg(out_dir)
                .spawn()
                .expect("Failed to copy the binary file.")
                .wait();
        }
    }
}

fn compile_rbpf(
    bpf_source_file: &str,
    binary_file: &Option<String>,
    out_dir: &str,
    elf_section_name: &str,
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

    compile_and_patch_rbpf_bytecode(file_name, &obj_file);


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
}

fn compile_and_patch_rbpf_bytecode(file_name: &str, obj_file: &str){
    // TODO: migrate those scripts to code for better flexibility.
    // The first compilation step involves using clang and llvm to compile
    // the eBPF bytecode exactly like it is done in case of the Linux kernel
    // eBPF programs.
    let _ = Command::new("bash")
        .arg("./scripts/compile.sh")
        .arg(file_name)
        .arg(obj_file)
        .spawn()
        .expect("Failed to compile the eBPF bytecode.")
        .wait();

    // TODO: make this step fully generic instead of manual hard-coded string
    // replacement
    // The second compilation step patches the bytecode to correct the
    // packet data offsets and replace the instructions so that the packet data
    // is loaded as 8-byte double words.
    let _ = Command::new("bash")
        .arg("./scripts/adjust-bytecode.sh")
        .arg(obj_file)
        .spawn()
        .expect("Failed to patch the eBPF bytecode.")
        .wait();
}

