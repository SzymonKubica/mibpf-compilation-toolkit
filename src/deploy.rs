use crate::{args::Action, compile::handle_compile, pull::handle_pull, sign::handle_sign};

pub async fn handle_deploy(args: &crate::args::Action) {
    if let Action::Deploy {
        bpf_source_file,
        target,
        out_dir,
        elf_section_name,
        host_network_interface,
        board_name,
        coaproot_dir,
        suit_storage_slot,
        riot_ipv6_addr,
        host_ipv6_addr,
    } = args
    {
        handle_compile(&Action::Compile {
            bpf_source_file: bpf_source_file.to_string(),
            target: target.to_string(),
            binary_file: Some("program.bin".to_string()),
            out_dir: out_dir.to_string(),
            elf_section_name: elf_section_name.to_string(),
        });

        handle_sign(&Action::Sign {
            host_network_interface: host_network_interface.to_string(),
            board_name: board_name.to_string(),
            coaproot_dir: coaproot_dir.to_string(),
            binary_name: "program.bin".to_string(),
            suit_storage_slot: *suit_storage_slot,
        });

        handle_pull(&Action::Pull {
            riot_ipv6_addr: riot_ipv6_addr.to_string(),
            host_ipv6_addr: host_ipv6_addr.to_string(),
            suit_manifest: format!("suit_manifest{}.signed", suit_storage_slot),
            host_network_interface: host_network_interface.to_string()
        })
        .await;
    } else {
        panic!("Invalid action args: {:?}", args);
    }
}
