use std::{
    fs::{self, File},
    io::Read,
};

use goblin::elf64::sym::{STB_GLOBAL, STT_FUNC, STT_OBJECT, STT_SECTION};

use crate::args::Action;

pub fn handle_relocate(args: &crate::args::Action) {
    if let Action::Relocate {
        source_object_file,
        binary_file,
    } = args
    {
        // Once the bytecode is patched and the offsets are adjusted correctly
        // we need to strip off the main program section from the object file.
        // This is because only this part is being used by the rbpf VM.

        // Read in the object file into the buffer.
        let mut f = File::open(&source_object_file).expect("no file found");
        let metadata = fs::metadata(&source_object_file).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        if let Ok(binary) = goblin::elf::Elf::parse(&buffer) {
            // First pass involves extracting the text, data, rodata sections and
            // the relocations.

            let mut text: Vec<u8> = vec![];
            let mut data: Vec<u8> = vec![];
            let mut rodata: Vec<u8> = vec![];

            for section in &binary.section_headers {
                let section_name = binary.strtab.get_at(section.sh_name);
                println!("Section name: {:?}", section_name);
                println!("Section header: {:?}", section);

                let maybe_relevant_section = match section_name {
                    Some(".text") => Some(&mut text),
                    Some(".data") => Some(&mut data),
                    Some(".rodata") => Some(&mut rodata),
                    _ => None,
                };

                if let Some(relevant_section) = maybe_relevant_section {
                    relevant_section.extend(
                        &buffer[section.sh_offset as usize
                            ..(section.sh_offset + section.sh_size) as usize],
                    );
                }

                if section.sh_type == goblin::elf::section_header::SHT_REL {
                    let offset = section.sh_offset as usize;
                    let size = section.sh_size as usize;
                    let relocations = goblin::elf::reloc::RelocSection::parse(
                        &buffer,
                        offset,
                        size,
                        false,
                        goblin::container::Ctx::default(),
                    )
                    .unwrap();
                    for reloc in relocations.iter() {
                        println!("Reloc: {:?}", reloc);
                    }
                }
            }

            println!("Text section:");
            print_bytes(&text);
            println!("Data section:");
            print_bytes(&data);
            println!("Read-only Data section:");
            print_bytes(&rodata);

            // Second pass
            // String literals used in e.g. calls to printf are loaded into the
            // .rodata.str.1 section, we need to move it over to the rodata section.
            // In order to perform relocations properly later on, we need to maintain
            // the map from the name of the additional rodata section to the offset
            // to it relative to the original rodata section.
            let mut str_section_offsets = std::collections::HashMap::new();

            for section in &binary.section_headers {
                if let Some(section_name) = binary.strtab.get_at(section.sh_name) {
                    if section_name.contains(".rodata.str") {
                        str_section_offsets.insert(section_name, rodata.len());
                        rodata.extend(
                            &buffer[section.sh_offset as usize
                                ..(section.sh_offset + section.sh_size) as usize],
                        );
                    }
                }
            }
            println!(
                "Additional read-only string sections: {:?}",
                str_section_offsets
            );

            // Maintains the offsets in the text section for all of the global
            // functions
            let mut symbol_map = std::collections::HashMap::new();

            for symbol in binary.syms.iter() {
                if symbol.st_type() == STT_FUNC && symbol.st_bind() == STB_GLOBAL {
                    let symbol_name = binary.strtab.get_at(symbol.st_name).unwrap();
                    println!("Found global function: {}", symbol_name);
                    let offset_within_text = symbol.st_value as usize;
                    symbol_map.insert(symbol_name, offset_within_text);
                }
            }

            let mut symbol_structs = vec![];

            // No we append all relocation symbols to the rodata section,
            for (name, text_offset) in symbol_map {
                let offset = rodata.len();
                let name_cstr = std::ffi::CString::new(name).unwrap();
                rodata.extend(name_cstr.to_bytes().iter());
                println!(
                    "Symbol {} with offset {} appended at {}",
                    name, text_offset, offset
                );
                // Added flags for compatiblity with rbpf
                let flags = 0;
                symbol_structs.push((offset, flags, text_offset));

            }

            // Handle relocations
            for section in &binary.section_headers {
                if section.sh_type == goblin::elf::section_header::SHT_REL {
                    let offset = section.sh_offset as usize;
                    let size = section.sh_size as usize;
                    let relocations = goblin::elf::reloc::RelocSection::parse(
                        &buffer,
                        offset,
                        size,
                        false,
                        goblin::container::Ctx::default(),
                    )
                    .unwrap();
                    for reloc in relocations.iter() {
                        println!("Reloc: {:?}", reloc);
                        if let Some(symbol) = binary.syms.get(reloc.r_sym) {
                            let section = binary.section_headers.get(symbol.st_shndx).unwrap();
                            if symbol.st_type() == STT_SECTION {
                                let name = binary.strtab.get_at(section.sh_name).unwrap();
                                println!(
                                    "relocation at instruction {} for section {} at {}",
                                    reloc.r_offset, name, symbol.st_value
                                )
                            } else {
                                let name = binary.strtab.get_at(symbol.st_name).unwrap();
                                let section_name = binary.strtab.get_at(section.sh_name).unwrap();
                                println!(
                                    "relocation at instruction {} for symbol {} in {} at {}",
                                    reloc.r_offset, name, section_name, symbol.st_value
                                )
                            }
                        }

                        patch_text(&mut text, &binary, reloc, &str_section_offsets);
                    }
                }
            }
        print_bytes(&text);
        }


    } else {
        panic!("Invalid action args: {:?}", args);
    }
}

#[repr(C, packed)]
struct Lddw {
    opcode: u8,
    registers: u8,
    offset: u16,
    immediate_l: u32,
    null1: u8,
    null2: u8,
    null3: u16,
    immediate_h: u32,
}

fn patch_text(
    text: &mut [u8],
    binary: &goblin::elf::Elf<'_>,
    reloc: goblin::elf::Reloc,
    str_section_offsets: &std::collections::HashMap<&str, usize>,
) {
    let symbol = binary.syms.get(reloc.r_sym).unwrap();
    let section = binary.section_headers.get(symbol.st_shndx).unwrap();
    let section_name = binary.strtab.get_at(section.sh_name).unwrap();
    let mut offset = 0;
    if symbol.st_type() == STT_SECTION {
        println!("section_name: {}", section_name);
        println!("registered section offsets: {:?}", str_section_offsets);
        if let Some(off) = str_section_offsets.get(section_name) {
            offset = *off;
        } else {
            return;
        }
    } else if symbol.st_type() == STT_OBJECT {
        offset = symbol.st_value as usize;
    } else if symbol.st_type() == STT_FUNC {
        // We don't do eny relocations in case of functions as they are handled
        // in a custom way by the VM (we append their relocation structs at the end of the binary
        // file)
        return;
    }

    const LDDWD_OPCODE: u32 = 0xB8;
    const LDDWR_OPCODE: u32 = 0xD8;
    const LDDW_OPCODE: u32 = 0x18;

    let opcode = if section_name.contains(".rodata.str") {
        LDDWR_OPCODE
    } else {
        LDDWD_OPCODE
    };

    if text[reloc.r_offset as usize] != LDDW_OPCODE as u8 {
        println!("No LDDW instruction at {}", reloc.r_offset);
        return;
    } else {
        let instruction = Vec::from(&text[reloc.r_offset as usize..reloc.r_offset as usize + 16]);
        println!(
            "Replacing {:?} at {} with {} at {}",
            instruction, reloc.r_offset, opcode, reloc.r_offset
        );
        //LDDW_STRUCT = struct.Struct('<BBHiBBHi')
        let mut instr: Lddw = unsafe { std::ptr::read(instruction.as_ptr() as *const _) };

        instr.opcode = opcode as u8;
        instr.immediate_l += offset as u32;

        let new_instruction =
            unsafe { std::slice::from_raw_parts(&instr as *const _ as *const u8, 16) };

        text[reloc.r_offset as usize..reloc.r_offset as usize + 16]
            .copy_from_slice(new_instruction);
    }
}

fn print_bytes(bytes: &[u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }
}
