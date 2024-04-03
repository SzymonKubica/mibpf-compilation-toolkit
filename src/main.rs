use args::Action;
use clap::Parser;
use compile::handle_compile;
use deploy::handle_deploy;
use execute::handle_execute;
use postprocessing::handle_relocate;
use pull::handle_pull;
use sign::handle_sign;

use crate::postprocessing::apply_postprocessing;

mod args;
mod compile;
mod deploy;
mod execute;
mod postprocessing;
mod pull;
mod sign;

extern crate clap;
extern crate coap;
extern crate env_logger;
extern crate internal_representation;
extern crate rbpf;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = args::Args::parse();

    let result = match &args.command {
        Action::Compile { .. } => handle_compile(&args.command),
        Action::Sign { .. } => handle_sign(&args.command),
        Action::Pull { .. } => handl_pull(&args.command),
        Action::Execute { .. } => hande_execute(&args.command),
        Action::Relocate { .. } => handle_relocate(&args.command),
        Action::Deploy { .. } => handle_deploy(&args.command).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_compile(args: &Action) -> Result<(), String> {
    let Action::Compile {
        bpf_source_file,
        binary_file,
        out_dir,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    compile::compile(bpf_source_file, binary_file, out_dir)
}

fn handle_sign(args: &Action) -> Result<(), String> {
    let Action::Sign {
        host_network_interface,
        board_name,
        coaproot_dir,
        binary_name,
        suit_storage_slot,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    sign::sign(
        host_network_interface,
        board_name,
        coaproot_dir,
        binary_name,
        suit_storage_slot as u32,
    )
}

fn handle_pull(args: &Action) -> Result<(), String> {
    let Action::Pull {
        riot_ipv6_addr,
        host_ipv6_addr,
        suit_manifest,
        host_network_interface,
        riot_network_interface,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    pull::pull(
        riot_ipv6_addr,
        host_ipv6_addr,
        suit_manifest,
        host_network_interface,
        riot_network_interface,
    )
    .await
}
fn handle_execute(args: &Action) -> Result<(), String> {
    let Action::Execute {
        riot_ipv6_addr,
        target,
        binary_layout,
        suit_storage_slot,
        host_network_interface,
        execution_model,
        helper_indices,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    let vm_target = TargetVM::from_str(target.as_str())?;
    let execution_model = ExecutionModel::from_str(execution_model)?;
    let binary_file_layout = binary_layout.as_str().parse::<BinaryFileLayout>()?;

    execute::execute(
        riot_ipv6_addr,
        target,
        binary_layout,
        suit_storage_slot,
        host_network_interface,
        execution_model,
        helper_indices,
    )
    .await
}

pub fn handle_relocate(args: &Action) -> Result<(), String> {
    let Action::Postprocessing {
        source_object_file,
        binary_file,
        binary_layout,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    let binary_layout = binary_layout.as_str().parse::<BinaryFileLayout>()?;

    let file_name = if let Some(binary_file) = binary_file {
        binary_file
    } else {
        "a.bin"
    };

    apply_postprocessing(source_object_file, binary_layout, file_name)
}

pub fn handle_deploy(args: &Action) -> Result<(), String> {
    let Action::Deploy {
        bpf_source_file,
        out_dir,
        host_network_interface,
        board_name,
        coaproot_dir,
        suit_storage_slot,
        riot_ipv6_addr,
        host_ipv6_addr,
        binary_layout,
        riot_network_interface,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    let binary_layout = binary_layout.as_str().parse::<BinaryFileLayout>()?;

    deploy::deploy(
        bpf_source_file,
        out_dir,
        host_network_interface,
        board_name,
        coaproot_dir,
        suit_storage_slot,
        riot_ipv6_addr,
        host_ipv6_addr,
        binary_layout,
        riot_network_interface,
    )
}
