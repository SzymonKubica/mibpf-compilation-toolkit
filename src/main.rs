use clap::Parser;
use compile::handle_compile;
use deploy::handle_deploy;
use execute::handle_execute;
use pull::handle_pull;
use sign::handle_sign;

mod args;
mod compile;
mod deploy;
mod execute;
mod pull;
mod sign;

extern crate clap;
extern crate coap;
extern crate rbpf;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    match &args.command {
        args::Action::Compile { .. } => handle_compile(&args.command),
        args::Action::Sign { .. } => handle_sign(&args.command),
        args::Action::Pull { .. } => handle_pull(&args.command).await,
        args::Action::Execute { .. } => handle_execute(&args.command).await,
        args::Action::Deploy { .. } => handle_deploy(&args.command).await,
        args::Action::EmulateExecution { .. } => handle_emulate(&args.command),
    }
}
