use clap::Parser;
use compile::handle_compile;
use deploy::handle_deploy;
use execute::handle_execute;
use pull::handle_pull;
use relocate::handle_relocate;
use sign::handle_sign;

mod args;
mod compile;
mod deploy;
mod execute;
mod internal_representation;
mod pull;
mod relocate;
mod sign;

extern crate clap;
extern crate coap;
extern crate env_logger;
extern crate rbpf;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = args::Args::parse();

    let result = match &args.command {
        args::Action::Compile { .. } => handle_compile(&args.command),
        args::Action::Sign { .. } => handle_sign(&args.command),
        args::Action::Pull { .. } => handle_pull(&args.command).await,
        args::Action::Execute { .. } => handle_execute(&args.command).await,
        args::Action::Deploy { .. } => handle_deploy(&args.command).await,
        args::Action::Relocate { .. } => handle_relocate(&args.command),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
