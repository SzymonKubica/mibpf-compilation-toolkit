
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

    bytecode_patching::perform_relocations(source_object_file, binary_file.clone(), *strip_debug)
}
