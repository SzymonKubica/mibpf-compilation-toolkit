use std::fs::File;
use std::io::Write;
use std::{path::PathBuf, process::Command};

fn main() {
    let file_name = std::env::args().nth(1).unwrap();

    let base_name = file_name
        .split(".")
        .nth(0)
        .expect("You need to provide the .c source file")
        .to_string();

    // The first compilation step involves using clang and llvm to compile
    // the eBPF bytecode exactly like it is done in case of the Linux kernel
    // eBPF programs.
    let _ = Command::new("bash")
        .arg("./scripts/compile.sh")
        .arg(base_name.clone())
        .spawn()
        .expect("Failed to compile the eBPF bytecode.")
        .wait();

    // The second compilation step patches the bytecode to correct the
    // packet data offsets and replace the instructions so that the packet data
    // is loaded as 8-byte double words.
    let _ = Command::new("bash")
        .arg("./scripts/adjust-bytecode.sh")
        .arg(base_name.clone())
        .spawn()
        .expect("Failed to patch the eBPF bytecode.")
        .wait();

    // Once the bytecode is patched and the offsets are adjusted correctly
    // we need to strip off the main program section from the object file.
    // This is because only this part is being used by the rbpf VM.

    let obj_file = format!("{}.o", base_name);
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
}
