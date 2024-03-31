use crate::args::Action;
/// Relocate subcommand is responsible for performing the post-processing of the
/// compiled eBPF bytecode before it can be loaded onto the target device. It
/// handles function relocations and read only data relocations.
pub fn handle_relocate(args: &crate::args::Action) -> Result<(), String> {
    let Action::Relocate {
        source_object_file,
        binary_file,
        strip_debug,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    if strip_debug {
        let _ = strip_binary(source_object_file, binary_file.as_ref());
        println!("Relocating the original binary");
        let mut buffer = read_file_as_bytes(source_object_file);
        let _ = relocate_in_place(&mut buffer);
        println!("Now relocating the stripped binary");
        let mut buffer = read_file_as_bytes(binary_file.as_ref().unwrap());
        return relocate_in_place(&mut buffer);
    }

    let elf_file = read_file_as_bytes(source_object_file);

    bytecode_patching::perform_relocations(elf_file);

    let mut f = File::create(file_name).unwrap();
    if log_enabled!(Level::Debug) {
        debug!("Generated binary:");
        print_bytes(&binary_data);
    }
    f.write_all(&binary_data).unwrap();
    Ok(())
}

fn read_file_as_bytes(source_object_file: &str) -> Vec<u8> {
    let mut f = File::open(&source_object_file).expect("File not found.");
    let metadata = fs::metadata(&source_object_file).expect("Unable to read file metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    buffer
}


pub fn strip_binary(source_object_file: &str, binary_file: Option<&String>) -> Result<(), String> {
    // strip bpf/helper-tests/out/gcoap.o -d -R .BTF -R .BTF.ext -o test
    let file_name = if let Some(binary_file) = binary_file {
        binary_file.clone()
    } else {
        "a.bin".to_string()
    };

    let result = Command::new("strip")
        .arg(source_object_file)
        .arg("-d")
        .arg("-R")
        .arg(".BTF")
        .arg("-R")
        .arg(".BTF.ext")
        .arg("-o")
        .arg(file_name)
        .spawn()
        .expect("Failed to compile the eBPF bytecode.")
        .wait();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to strip the binary: {}", e)),
    }
}


