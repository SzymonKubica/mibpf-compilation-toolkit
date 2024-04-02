use std::{
    fs::{self, File},
    io::{Read, Write as _},
    process::Command,
};

use bytecode_patching::{
    assemble_binary, assemble_femtocontainer_binary, debug_print_program_bytes, extract_section,
    resolve_relocations,
};
use internal_representation::BinaryFileLayout;
use log::{debug, log_enabled, Level};

use crate::args::Action;

// This module is responsible for applying different post-processing steps
// to the input ELF file to transform it into a corresponding binary layout
// that the VM expects to when loading the program.

pub fn apply_postprocessing(
    source_object_file: &str,
    binary_layout: BinaryFileLayout,
    output_file_name: &str,
) -> Result<(), String> {
    // When we want to perform relocations on the actual target device, we
    // only need to strip off the redundant information from the object file.
    if binary_layout == BinaryFileLayout::RawObjectFile {
        return strip_binary(&source_object_file, Some(&output_file_name.to_string()));
    }

    let processed_program_bytes = match binary_layout {
        BinaryFileLayout::OnlyTextSection => {
            let program_bytes = read_bytes_from_file(source_object_file);
            let text_section_bytes = extract_section(".text", &program_bytes)?;
            Vec::from(text_section_bytes)
        }
        BinaryFileLayout::FunctionRelocationMetadata => {
            let program_bytes = read_bytes_from_file(source_object_file);
            let relocated_program = assemble_binary(&program_bytes)?;
            relocated_program
        }
        BinaryFileLayout::FemtoContainersHeader => {
            let program_bytes = read_bytes_from_file(source_object_file);
            let relocated_program = assemble_femtocontainer_binary(&program_bytes)?;
            relocated_program
        }
        BinaryFileLayout::RawObjectFile => {
            unreachable!()
        }
    };

    write_binary(&processed_program_bytes, output_file_name);
    Ok(())
}

fn write_binary(bytes: &[u8], destination: &str) {
    let mut f = File::create(destination).unwrap();
    f.write_all(bytes).unwrap();
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

    if *strip_debug {
        let _ = strip_binary(source_object_file, binary_file.as_ref());
        println!("Relocating the original binary");
        let mut buffer = read_bytes_from_file(source_object_file);
        let _ = resolve_relocations(&mut buffer);
        println!("Now relocating the stripped binary");
        let mut buffer = read_bytes_from_file(binary_file.as_ref().unwrap());
        return resolve_relocations(&mut buffer);
    }

    let elf_file = read_bytes_from_file(source_object_file);

    let relocated_file = bytecode_patching::assemble_femtocontainer_binary(&elf_file)?;

    let file_name = if let Some(binary_file) = binary_file {
        binary_file.clone()
    } else {
        "a.bin".to_string()
    };

    let mut f = File::create(file_name).unwrap();
    if log_enabled!(Level::Debug) {
        debug!("Generated binary:");
        debug_print_program_bytes(&relocated_file);
    }
    f.write_all(&relocated_file).unwrap();
    Ok(())
}

pub fn read_bytes_from_file(source_object_file: &str) -> Vec<u8> {
    let mut f = File::open(&source_object_file).expect("File not found.");
    let metadata = fs::metadata(&source_object_file).expect("Unable to read file metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    buffer
}

/// Uses the strip command to remove all of the debug and .BTF info from the
/// ELF object file. It is required in order to decrease the binary size so that
/// it can be sent directly to the target device where the relocations can be
/// performed.
pub fn strip_binary(source_object_file: &str, binary_file: Option<&String>) -> Result<(), String> {
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
