use std::process::Command;

use log::debug;

use crate::{
    args::Action,
    internal_representation::{BinaryFileLayout, TargetVM, VMExecutionRequestMsg},
};

pub struct VMConfiguration {
    pub vm_target: TargetVM,
    pub binary_layout: BinaryFileLayout,
    pub suit_slot: u8,
}

impl VMConfiguration {
    pub fn new(vm_target: TargetVM, binary_layout: BinaryFileLayout, suit_slot: u8) -> Self {
        VMConfiguration {
            vm_target,
            binary_layout,
            suit_slot,
        }
    }

    /// Encodes the VM configuration into a u8. The reason we need this is that
    /// RIOT message passing IPC infrastructure limits the size of the transported
    /// messages to 64 bits. In order to fully specify a given VM execution,
    /// we need all fields of the VMConfiguration struct and the metadata specifying
    /// which helper functions the VM is allowed to call. Encoding the configuration
    /// as a single u8 allows us to use the remaining bits to specify the helper
    /// metadata.
    ///
    /// The encoding is as follows:
    /// - The least significant bit specifies whether we should use the rbpf
    /// or the FemtoContainers VM. 0 corresponds to rbpf and 1 to FemtoContainers.
    /// - The next bit specifies the SUIT storage slot storing the eBPF program
    /// bytecode. There are only two available slots provided by RIOT so a single
    /// bit is sufficient.
    /// - The remaining bits are used to encode the binary layout that the VM
    /// should expect in the loaded program bytecode. Currently there are only 4
    /// options so 2 bits are sufficient. This can be adapted in the future.
    pub fn encode(&self) -> u8 {
        let mut encoding: u8 = 0;
        encoding |= self.vm_target as u8;
        encoding |= (self.suit_slot as u8) << 1;
        encoding |= (self.binary_layout as u8) << 2;
        encoding
    }

    /// Decodes the VM configuration according to the encoding specified above.
    pub fn decode(encoding: u8) -> Self {
        VMConfiguration {
            vm_target: TargetVM::from(encoding & 0b1),
            suit_slot: (encoding >> 1) & 0b1,
            binary_layout: BinaryFileLayout::from((encoding >> 2) & 0b11),
        }
    }
}


pub async fn handle_execute(args: &crate::args::Action) -> Result<(), String> {
    let Action::Execute {
        riot_ipv6_addr: riot_ipv6,
        target,
        binary_layout,
        suit_storage_slot,
        host_network_interface: host_net_if,
        execute_on_coap,
        helper_indices,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };
    let vm_target = TargetVM::from(target.as_str());

    let binary_layout = BinaryFileLayout::from(binary_layout.as_str());

    let request = VMExecutionRequestMsg {
        configuration: VMConfiguration::new(vm_target, binary_layout, *suit_storage_slot as u8).encode(),
        available_helpers: encode(helper_indices)

    };

    let url = if !*execute_on_coap {
        format!("coap://[{}%{}]/vm/spawn", riot_ipv6, host_net_if)
    } else {
        format!("coap://[{}%{}]/vm/exec/coap-pkt", riot_ipv6, host_net_if,)
    };

    debug!("Sending a request to the url: {}", url);

    let payload = serde_json::to_string(&request).unwrap();
    println!("{}", payload);

    let Ok(_) = Command::new("aiocoap-client")
        .arg("-m")
        .arg("POST")
        .arg(url.clone())
        .arg("--payload")
        .arg(&payload)
        .spawn()
        .expect("Failed to send the request.")
        .wait()
    else {
        return Err(format!("Failed to send request payload: {}", payload));
    };

    Ok(())
}

fn encode(available_indices: &[u8]) -> [u8; 3] {
    let mut encoding = [0; 3];
    for i in available_indices {
        // The first 8 helpers are configured by the first u8, the next
        // by the second one and so on.
        let bucket = (i / 8) as usize;
        encoding[bucket] |= 1 << (i % 8);
    }
    return encoding;
}

