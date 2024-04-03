use args::Action;
use clap::Parser;
use compile::handle_compile;
use deploy::handle_deploy;
use execute::handle_execute;
use postprocessing::handle_relocate;
use pull::handle_pull;
use sign::handle_sign;

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
        Action::Compile {
            bpf_source_file,
            binary_file,
            out_dir,
        } => handle_compile(bpf_source_file, binary_file, out_dir),
        Action::Sign {
            host_network_interface,
            board_name,
            coaproot_dir,
            binary_name,
            suit_storage_slot,
        } => handle_sign(
            host_network_interface,
            board_name,
            coaproot_dir,
            binary_name,
            suit_storage_slot as u32,
        ),
        Action::Pull {
            riot_ipv6_addr,
            host_ipv6_addr,
            suit_manifest,
            host_network_interface,
            riot_network_interface,
        } => {
            handle_pull(
                riot_ipv6_addr,
                host_ipv6_addr,
                suit_manifest,
                host_network_interface,
                riot_network_interface,
            )
            .await
        }
        Action::Execute {
            riot_ipv6_addr,
            target,
            binary_layout,
            suit_storage_slot,
            host_network_interface,
            execution_model,
            helper_indices,
        } => {
            let Ok(vm_target) = TargetVM::from_str(target.as_str()) else {
                return Err(format!("Invalid subcommand args: {:?}", args));
            };

            let execution_model = ExecutionModel::from_str(execution_model)?;
            let binary_file_layout = binary_layout.as_str().parse::<BinaryFileLayout>().unwrap();
            handle_execute(
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
        args::Action::Deploy { .. } => handle_deploy(&args.command).await,
        args::Action::Relocate { .. } => handle_relocate(&args.command),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
