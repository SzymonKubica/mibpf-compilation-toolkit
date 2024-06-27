//! This crate provides utility functions for compiling, deploying and executing
//! eBPF programs on embedded devices running RIOT-OS with the micro_bpf implementation
//! of the virtual machine.

extern crate clap;
extern crate coap;
extern crate env_logger;
extern crate rbpf;
extern crate micro_bpf_common;

mod args;
mod compile;
mod deploy;
mod execute;
mod pull;
mod postprocessing;
mod sign;
mod environment;

pub use compile::compile;
pub use deploy::deploy;
pub use execute::execute;
pub use pull::pull;
pub use postprocessing::apply_postprocessing;
pub use sign::sign;

pub use environment::{Environment, load_env};

