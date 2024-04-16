use std::{
    fs::{self, File},
    io::{Read, Write as _},
    process::Command,
};

use mibpf_common::{BinaryFileLayout, HelperAccessVerification};
use mibpf_elf_utils::{
    assemble_binary_specifying_helpers, assemble_femtocontainer_binary, extract_section,
};

// This module is responsible for applying different post-processing steps
// to the input ELF file to transform it into a corresponding binary layout
// that the VM expects to when loading the program.
pub fn apply_postprocessing(
    source_object_file: &str,
    binary_layout: BinaryFileLayout,
    output_file_name: &str,
    helper_indices: Vec<u8>,
    helper_access_verification: HelperAccessVerification,
) -> Result<(), String> {
    let processed_program_bytes = match binary_layout {
        BinaryFileLayout::OnlyTextSection => {
            let program_bytes = read_bytes_from_file(source_object_file);
            let text_section_bytes = extract_section(".text", &program_bytes)?;
            Vec::from(text_section_bytes)
        }
        BinaryFileLayout::ExtendedHeader => {
            let program_bytes = read_bytes_from_file(source_object_file);
            let relocated_program =
                assemble_binary_specifying_helpers(&program_bytes, helper_indices.clone())?;
            relocated_program
        }
        BinaryFileLayout::FemtoContainersHeader => {
            let program_bytes = read_bytes_from_file(source_object_file);
            let relocated_program = assemble_femtocontainer_binary(&program_bytes)?;
            relocated_program
        }
        BinaryFileLayout::RawObjectFile => {
            strip_binary(&source_object_file, Some(&output_file_name.to_string()))?;
            read_bytes_from_file(output_file_name)
        }
    };

    if helper_access_verification == HelperAccessVerification::AheadOfTime {
        // We first need to map our state to the structures that rbpf understands
        let helper_idxs = helper_indices
            .iter()
            .map(|id| *id as u32)
            .collect::<Vec<u32>>();
        let interpreter = map_interpreter(binary_layout);
        rbpf::check_helpers(&processed_program_bytes, &helper_idxs, interpreter)
            .map_err(|e| format!("Error when checking helper function access: {:?}", e))?;
    }

    write_binary(&processed_program_bytes, output_file_name)
}

pub fn map_interpreter(layout: BinaryFileLayout) -> rbpf::InterpreterVariant {
    match layout {
        BinaryFileLayout::FemtoContainersHeader => rbpf::InterpreterVariant::FemtoContainersHeader,
        BinaryFileLayout::ExtendedHeader => rbpf::InterpreterVariant::ExtendedHeader,
        BinaryFileLayout::RawObjectFile => rbpf::InterpreterVariant::RawObjectFile,
        BinaryFileLayout::OnlyTextSection => rbpf::InterpreterVariant::Default,
    }
}

fn write_binary(bytes: &[u8], destination: &str) -> Result<(), String> {
    let Ok(mut f) = File::create(destination) else {
        return Err(format!("Failed to create the file: {}", destination));
    };
    f.write_all(bytes)
        .map_err(|e| format!("Error when writing to a file: {}", e))
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
