use std::{
    fs::{self, File},
    io::Read,
};

use goblin::elf64::sym::{STB_GLOBAL, STT_FUNC, STT_SECTION};

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

            // No we append all relocation symbols to the rodata section,
            for (name, text_offset) in symbol_map {
                let offset = rodata.len();
                let name_cstr = std::ffi::CString::new(name).unwrap();
                rodata.extend(name_cstr.to_bytes().iter());
                println!(
                    "Symbol {} with offset {} appended at {}",
                    name, text_offset, offset
                );
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

                        patch_text(&mut text, &binary, reloc, str_section_offsets);
                    }
                }
            }
        }

        /*

        // Now handle relocations
        /*
        *         if relocations:
           for relocation in relocations.iter_relocations():
               logging.debug(relocation.entry)
               entry = relocation.entry
               symbol = symbols.get_symbol(entry['r_info_sym'])
               if symbol.entry['st_info']['type'] == 'STT_SECTION':
                   name = elffile.get_section(symbol.entry['st_shndx']).name
                   logging.info(f"relocation at instruction {hex(entry['r_offset'])} for section {name} at offset {symbol.entry.st_value}")
               else:
                   name = symbol.name
                   section = elffile.get_section(symbol.entry.st_shndx)
                   logging.info(f"relocation at instruction {hex(entry['r_offset'])} for symbol {name} in {section.name} at {symbol.entry.st_value}")

        */

        if let Some(relocations) = relocations {
            println!("Relocations: {}", relocations);
            let mut offset;
            elf::relocation::Rel::parse_at(BigEndian, Class::ELF32, &mut offset, relocations.data);
        }
        */
    } else {
        panic!("Invalid action args: {:?}", args);
    }
}

fn patch_text(
    text: &[u8],
    binary: &goblin::elf::Elf<'_>,
    reloc: goblin::elf::Reloc,
    str_section_offsets: std::collections::HashMap<&str, usize>,
) {
}

fn print_bytes(bytes: &[u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }
}
