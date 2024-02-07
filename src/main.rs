use clap::Parser;
use compile::handle_compile;
use execute::handle_execute;
use pull::handle_pull;
use sign::handle_sign;
mod args;
mod compile;
mod execute;
mod pull;
mod sign;

extern crate clap;
extern crate rbpf;

fn main() {
    let args = args::Args::parse();

    match &args.command {
        args::Action::Compile { .. } => handle_compile(&args.command),
        args::Action::Sign { .. } => handle_sign(&args.command),
        args::Action::Pull { .. } => handle_pull(&args.command),
        args::Action::Execute { .. } => handle_execute(&args.command),
    }
}
