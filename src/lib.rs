mod args;
mod compile;
mod deploy;
mod execute;
mod pull;
mod postprocessing;
mod sign;

extern crate clap;
extern crate coap;
extern crate env_logger;
extern crate rbpf;
extern crate internal_representation;

pub use compile::handle_compile;
pub use deploy::handle_deploy;
pub use execute::handle_execute;
pub use pull::handle_pull;
pub use postprocessing::handle_relocate;
pub use sign::handle_sign;
pub use args::Action;

