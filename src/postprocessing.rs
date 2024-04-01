use internal_representation::BinaryFileLayout;

use crate::args::Action;
// This module is responsible for applying different post-processing steps
// to the input ELF file to transform it into a corresponding binary layout
// that the VM expects to when loading the program.

pub fn apply_postprocessing(source_object_file: &str, binary_layout: BinaryFileLayout) -> Result<(), String> {
    if binary_file_layout != BinaryFileLayout::FemtoContainersHeader {
        // In this case we need to produce the binary ourselves and place it in
        // the coaproot directory. This is because the binary produced by RIOT
        // is not suitable for the specified binary layout.
        // We need to place the binary in the coaproot directory so that the
        // signing script can find it.

        match binary_file_layout {
            BinaryFileLayout::OnlyTextSection => {
                let bytes = extract_text_section(bpf_source_file, out_dir);
                write_binary(&bytes, "program.bin");
            }
            BinaryFileLayout::FunctionRelocationMetadata => {
                let bytes = get_relocated_binary(bpf_source_file, out_dir);
                write_binary(&bytes, "program.bin");
            }
            BinaryFileLayout::RawObjectFile => {
                let object_file = get_object_file_name(bpf_source_file, out_dir);
                let _ = strip_binary(&object_file, Some(&"program.bin".to_string()));
            }
            BinaryFileLayout::FemtoContainersHeader => unreachable!(),
        }
    }
    Ok(())
}

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


