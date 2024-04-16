extern crate clap;
extern crate coap;
extern crate env_logger;
extern crate mibpf_common;
extern crate rbpf;

mod args;
mod compile;
mod deploy;
mod environment;
mod execute;
mod postprocessing;
mod pull;
mod sign;

use std::str::FromStr;

use args::Action;
use clap::Parser;
use compile::compile;
use deploy::deploy;
use environment::load_env;
use execute::execute;
use mibpf_common::{
    BinaryFileLayout, ExecutionModel, HelperAccessListSource, HelperAccessVerification, TargetVM,
};
use postprocessing::apply_postprocessing;
use pull::pull;
use sign::sign;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = args::Args::parse();

    let use_env = args.use_env;

    let result = match &args.command {
        Action::Compile { .. } => handle_compile(&args.command, use_env),
        Action::Postprocessing { .. } => handle_postprocessing(&args.command),
        Action::Sign { .. } => handle_sign(&args.command, use_env),
        Action::Pull { .. } => handle_pull(&args.command, use_env).await,
        Action::Execute { .. } => handle_execute(&args.command, use_env).await,
        Action::Deploy { .. } => handle_deploy(&args.command, use_env).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_compile(args: &Action, use_env: bool) -> Result<(), String> {
    let Action::Compile {
        bpf_source_file,
        binary_file,
        out_dir,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    if use_env {
        let env = load_env();

        return compile(bpf_source_file, binary_file.as_deref(), &env.out_dir);
    }

    compile(bpf_source_file, binary_file.as_deref(), out_dir)
}

fn handle_sign(args: &Action, use_env: bool) -> Result<(), String> {
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

    if use_env {
        let env = load_env();

        return sign(
            &env.host_net_if,
            &env.board_name,
            &env.coap_root_dir,
            binary_name,
            *suit_storage_slot as usize,
            None,
        );
    }

    sign(
        host_network_interface,
        board_name,
        coaproot_dir,
        binary_name,
        *suit_storage_slot as usize,
        None,
    )
}

async fn handle_pull(args: &Action, use_env: bool) -> Result<(), String> {
    let Action::Pull {
        riot_ipv6_addr,
        host_ipv6_addr,
        suit_manifest,
        host_network_interface,
        riot_network_interface,
        target,
        binary_layout,
        suit_storage_slot,
        helper_indices,
        helper_access_verification,
        helper_access_list_source,
        erase,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    let target_vm = TargetVM::from_str(target.as_str())?;
    let binary_file_layout = binary_layout.as_str().parse::<BinaryFileLayout>()?;
    let helper_access_verification =
        HelperAccessVerification::from_str(helper_access_verification.as_str())?;
    let helper_access_list_source =
        HelperAccessListSource::from_str(helper_access_list_source.as_str())?;

    if use_env {
        let env = load_env();

        return pull(
            &env.riot_instance_ip,
            &env.host_ip,
            suit_manifest,
            &env.host_net_if,
            &env.riot_instance_net_if,
            target_vm,
            binary_file_layout,
            *suit_storage_slot as usize,
            helper_access_verification,
            helper_access_list_source,
            helper_indices,
            *erase,
        )
        .await;
    }

    pull(
        riot_ipv6_addr,
        host_ipv6_addr,
        suit_manifest,
        host_network_interface,
        riot_network_interface,
        target_vm,
        binary_file_layout,
        *suit_storage_slot as usize,
        helper_access_verification,
        helper_access_list_source,
        helper_indices,
        *erase,
    )
    .await
}
async fn handle_execute(args: &Action, use_env: bool) -> Result<(), String> {
    let Action::Execute {
        riot_ipv6_addr,
        target,
        binary_layout,
        suit_storage_slot,
        host_network_interface,
        execution_model,
        helper_indices,
        helper_access_verification,
        helper_access_list_source,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    let target_vm = TargetVM::from_str(target.as_str())?;
    let execution_model = ExecutionModel::from_str(execution_model)?;
    let binary_file_layout = binary_layout.as_str().parse::<BinaryFileLayout>()?;
    let helper_access_verification =
        HelperAccessVerification::from_str(helper_access_verification.as_str())?;
    let helper_access_list_source =
        HelperAccessListSource::from_str(helper_access_list_source.as_str())?;

    let response = if use_env {
        let env = load_env();
        execute(
            &env.riot_instance_ip,
            target_vm,
            binary_file_layout,
            *suit_storage_slot as usize,
            &env.host_net_if,
            execution_model,
            helper_access_verification,
            helper_access_list_source,
            helper_indices,
        )
        .await?
    } else {
        execute(
            riot_ipv6_addr,
            target_vm,
            binary_file_layout,
            *suit_storage_slot as usize,
            host_network_interface,
            execution_model,
            helper_access_verification,
            helper_access_list_source,
            helper_indices,
        )
        .await?
    };

    println!("Response received: \n{}", response);

    Ok(())
}

fn handle_postprocessing(args: &Action) -> Result<(), String> {
    let Action::Postprocessing {
        source_object_file,
        binary_file,
        binary_layout,
        helper_indices,
        helper_access_verification,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    let binary_layout = binary_layout.as_str().parse::<BinaryFileLayout>()?;
    let helper_access_verification =
        HelperAccessVerification::from_str(helper_access_verification.as_str())?;

    let file_name = if let Some(binary_file) = binary_file {
        binary_file
    } else {
        "a.bin"
    };

    apply_postprocessing(
        source_object_file,
        binary_layout,
        file_name,
        helper_indices.to_vec(),
        helper_access_verification,
    )
}

async fn handle_deploy(args: &Action, use_env: bool) -> Result<(), String> {
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
        helper_indices,
        helper_access_verification,
        helper_access_list_source,
        target,
        erase,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };

    let target_vm = TargetVM::from_str(target.as_str())?;
    let binary_layout = binary_layout.as_str().parse::<BinaryFileLayout>()?;
    let helper_access_verification =
        HelperAccessVerification::from_str(helper_access_verification.as_str())?;
    let helper_access_list_source =
        HelperAccessListSource::from_str(helper_access_list_source.as_str())?;

    if use_env {
        let env = environment::load_env();

        return deploy(
            bpf_source_file,
            out_dir,
            target_vm,
            binary_layout,
            &env.coap_root_dir,
            *suit_storage_slot as usize,
            &env.riot_instance_net_if,
            &env.riot_instance_ip,
            &env.host_net_if,
            &env.host_ip,
            &env.board_name,
            None,
            helper_indices.to_vec(),
            helper_access_verification,
            helper_access_list_source,
            *erase,
        )
        .await;
    }

    deploy(
        bpf_source_file,
        out_dir,
        target_vm,
        binary_layout,
        coaproot_dir,
        *suit_storage_slot as usize,
        riot_network_interface,
        riot_ipv6_addr,
        host_network_interface,
        host_ipv6_addr,
        board_name,
        None,
        helper_indices.to_vec(),
        helper_access_verification,
        helper_access_list_source,
        *erase,
    )
    .await
}
