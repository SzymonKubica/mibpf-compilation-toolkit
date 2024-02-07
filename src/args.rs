use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Action {
    /// Compile the eBPF program
    Compile {
        /// Name of the file containing the eBPF source code.
        #[arg(long)]
        bpf_source_file: String,

        /// Target version of the eBPF vm. Available options: Femto-Containers, rBPF
        #[arg(long, default_value_t = String::from("rBPF"))]
        target: String,

        /// The name of the file resulting from the compilation
        #[arg(long, default_value_t = String::from("a.bin"))]
        output_file: String,
    },

    /// Sign the eBPF binary for SUIT update protocol. Generates  the manifest,
    /// signs it and places all files in the CoAP fileserver root directory
    Sign {
        /// Network interface of the machine hosting the CoAP fileserver.
        /// Used to find the IPv6 address of the fileserver.
        #[arg(long, default_value_t = String::from("wlan0"))]
        host_network_interface: String,

        /// Name of the target microcontroller board.
        #[arg(long, default_value_t = String::from("nucleo-f439zi"))]
        board_name: String,

        /// Name of the coaproot directory from the CoAP fileserver will serve
        /// the files. The signed binary and manifest will be placed there
        #[arg(long, default_value_t = String::from("coaproot"))]
        coaproot_dir: String,

        /// Name of the binary to sign and place in the CoAP fileserver root
        /// directory.
        #[arg(long, default_value_t = String::from("a.bin"))]
        binary_name: String,
    },

    /// Sends a request to the RIOT instance to fetch the new signed binary
    /// and load it into the specified SUIT storage slot.
    Pull {},
    /// Sends a request to the RIOT instance to execute the loaded eBPF bytecode
    /// from a specified SUIT storage slot.
    Execute {},
}

/// Tools for compiling, signing, loading and executing eBPF programs for
/// mibpf.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The command that is to be performed. Available options: Compile, Sign,
    /// Pull, Execute.
    #[command(subcommand)]
    pub command: Action,
}
