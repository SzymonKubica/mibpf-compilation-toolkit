use clap::Parser;
mod args;
mod compile;
mod execute;
mod pull;
mod sign;

extern crate clap;
extern crate rbpf;

use crate::compile::{handle_compile, VmTarget};

fn main() {
    let args = args::Args::parse();

    match &args.command {
        args::Action::Compile {
            bpf_source_file,
            target,
            output_file,
            elf_section_name,
            test_execution,
        } => {
            let vm_target = VmTarget::from(target.clone());
            handle_compile(bpf_source_file, vm_target, output_file, elf_section_name, *test_execution)
        }

        args::Action::Sign {
            host_network_interface,
            board_name,
            coaproot_dir,
            binary_name,
        } => {
        }
        args::Action::Pull {} => todo!(),
        args::Action::Execute {} => todo!(),
    }
}
