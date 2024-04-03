extern crate clap;
extern crate coap;
extern crate env_logger;
extern crate internal_representation;
extern crate rbpf;

mod args;
mod compile;
mod deploy;
mod execute;
mod postprocessing;
mod pull;
mod sign;

use std::str::FromStr;

use args::Action;
use clap::Parser;
use compile::compile;
use deploy::deploy;
use execute::execute;
use internal_representation::{BinaryFileLayout, ExecutionModel, TargetVM};
use postprocessing::apply_postprocessing;
use pull::pull;
use sign::sign;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = args::Args::parse();

    let result = match &args.command {
        Action::Compile { .. } => handle_compile(&args.command),
        Action::Postprocessing { .. } => handle_postprocessing(&args.command),
        Action::Sign { .. } => handle_sign(&args.command),
        Action::Pull { .. } => handle_pull(&args.command).await,
        Action::Execute { .. } => handle_execute(&args.command).await,
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

    compile(bpf_source_file, binary_file.as_deref(), out_dir)
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

    sign(
        host_network_interface,
        board_name,
        coaproot_dir,
        binary_name,
        *suit_storage_slot as usize,
    )
}

async fn handle_pull(args: &Action) -> Result<(), String> {
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

    pull(
        riot_ipv6_addr,
        host_ipv6_addr,
        suit_manifest,
        host_network_interface,
        riot_network_interface,
    )
    .await
}
async fn handle_execute(args: &Action) -> Result<(), String> {
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

    let target_vm = TargetVM::from_str(target.as_str())?;
    let execution_model = ExecutionModel::from_str(execution_model)?;
    let binary_file_layout = binary_layout.as_str().parse::<BinaryFileLayout>()?;

    execute(
        riot_ipv6_addr,
        target_vm,
        binary_file_layout,
        *suit_storage_slot as usize,
        host_network_interface,
        execution_model,
        helper_indices,
    )
    .await
}

fn handle_postprocessing(args: &Action) -> Result<(), String> {
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

async fn handle_deploy(args: &Action) -> Result<(), String> {
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

    deploy(
        bpf_source_file,
        out_dir,
        host_network_interface,
        board_name,
        coaproot_dir,
        *suit_storage_slot as usize,
        riot_ipv6_addr,
        host_ipv6_addr,
        binary_layout,
        riot_network_interface,
    )
    .await
}
