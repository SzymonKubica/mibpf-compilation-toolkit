use std::path::PathBuf;

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

        let path = PathBuf::from(source_object_file);
        let file = match elf::File::open_path(&path) {
            Ok(f) => f,
            Err(e) => panic!("Error: {:?}", e),
        };

        for section in &file.sections {
            println!("{:?}", section.shdr);
        }

    } else {
        panic!("Invalid action args: {:?}", args);
    }
}
