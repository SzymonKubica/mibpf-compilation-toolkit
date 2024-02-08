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

        /// Optional override for the name of the file resulting from the compilation
        /// It defaults to <source_file>.bin
        #[arg(long)]
        binary_file: Option<String>,

        /// Directory for the object files
        #[arg(long, default_value_t = String::from("./out"))]
        out_dir: String,

        /// Name of the elf section of the main function in the eBPF program.
        #[arg(long, default_value_t = String::from(".main"))]
        elf_section_name: String,

        /// Controlls whether the bytecode is executed in a native vm after compilation.
        #[arg(long, default_value_t = false)]
        test_execution: bool,
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

        /// SUIT storage slot (0 or 1) where the signed binary blob is intended
        /// bo be loaded.
        #[arg(long, short, default_value_t = 0)]
        suit_storage_slot: i32,
    },

    /// Sends a request to the RIOT instance to fetch the new signed binary
    /// and load it into the specified SUIT storage slot.
    Pull {
        /// IPv6 address of the RIOT instance.
        #[arg(long)]
        riot_ipv6_addr: String,

        /// IPv6 address of the desktop machine hosting the CoAP fileserver with
        /// the binary blob and SUIT manifest.
        #[arg(long)]
        host_ipv6_addr: String,

        /// Name of the signed SUIT manifest file present in the CoAP fileserver
        /// root directory.
        # [arg(long, default_value_t = String::from("suit_manifest.signed"))]
        suit_manifest: String,
    },
    /// Compiles, signs and initiates firmware pull in one step.
    Deploy {
        /// Name of the file containing the eBPF source code.
        #[arg(long)]
        bpf_source_file: String,

        /// Target version of the eBPF vm. Available options: Femto-Containers, rBPF
        #[arg(long, default_value_t = String::from("rBPF"))]
        target: String,

        /// Directory for the object files
        #[arg(long, default_value_t = String::from("./out"))]
        out_dir: String,

        /// Name of the elf section of the main function in the eBPF program.
        /// Only applies for the rbpf code snippets.
        #[arg(long, default_value_t = String::from(".main"))]
        elf_section_name: String,

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

        /// SUIT storage slot (0 or 1) where the signed binary blob is intended
        /// bo be loaded.
        #[arg(long, short, default_value_t = 0)]
        suit_storage_slot: i32,

        /// IPv6 address of the RIOT instance.
        #[arg(long)]
        riot_ipv6_addr: String,

        /// IPv6 address of the desktop machine hosting the CoAP fileserver with
        /// the binary blob and SUIT manifest.
        #[arg(long)]
        host_ipv6_addr: String,
    },
    /// Sends a request to the RIOT instance to execute the loaded eBPF bytecode
    /// from a specified SUIT storage slot.
    Execute {
        /// IPv6 address of the RIOT instance.
        #[arg(long)]
        riot_ipv6_addr: String,

        /// Target version of the eBPF vm. Available options: Femto-Containers, rBPF
        #[arg(long, default_value_t = String::from("rBPF"))]
        target: String,

        /// SUIT storage slot (0 or 1) where the signed binary blob is intended
        /// bo be loaded.
        #[arg(long, short, default_value_t = 0)]
        suit_storage_slot: i32,
    },
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
