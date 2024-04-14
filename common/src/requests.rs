use core::num::ParseIntError;

use alloc::{format, string::String, vec::Vec};
use serde::{Deserialize, Serialize};

/// Encoded transfer object representing a request to start a given execution
/// of the eBPF VM. It contains the encoded configuration of the VM and a vector
/// of the helper functions that are allowed to be called by the VM.
///
/// We need to encode the configuration because this struct is used to send the
/// serialized payload of our request using the CoAP network protocol. Because
/// of the packet size constraints, we cannot afford to specify all configuration
/// options as separate keys in the json request object.
///
/// # Example
/// ```
/// use mibpf_common::{VMConfiguration, VMExecutionRequestMsg, TargetVM, BinaryFileLayout};
///
/// // List of helper functions
/// let helpers = vec![0, 1, 2, 3];
///
/// // We specify the configuration settings
/// let suit_storage_slot = 0;
/// let binary_layout = BinaryFileLayout::RawObjectFile;
/// let config = VMConfiguration::new(TargetVM::Rbpf, binary_layout, suit_storage_slot);
///
/// // and construct the request message
/// let request = VMExecutionRequestMsg {
///     configuration: config.encode(),
///     allowed_helpers: helpers,
/// };
///
/// ```
#[derive(Clone, Debug)]
#[repr(C)]
pub struct VMExecutionRequestMsg {
    /// Encoded representation of the VM config. See [`crate::enumerations::VMConfiguration`]
    /// for more details.
    pub configuration: u8,
    /// A list of helper functions that the program executing in the VM should be
    /// allowed to call. Each helper function is represented by its index, i.e.
    /// the value that is used as the immediate operand of the CALL instruction
    /// that is used by the VM to call a given helper function. Those indices are
    /// represented by u8 values, which means that currently we can have up to
    /// 256 helper functions.
    pub allowed_helpers: Vec<u8>,
}

pub type HelperFunctionEncoding = String;

impl VMExecutionRequestMsg {
    pub fn new(configuration: u8, allowed_helpers: Vec<u8>) -> Self {
        VMExecutionRequestMsg {
            configuration,
            allowed_helpers,
        }
    }

    /// Because of the request entity size constraints when sending the CoAP packets
    /// we need to encode the request message into a compact format to allow for
    /// specifying the highest possible number of helper functions.
    ///
    /// The maximum supported length of the string request payload is 110
    /// (measured empirically - the aiocoap documentation doesn't say anything about it)
    ///
    /// Because of this limitation, we encode the request message as a string
    /// of concatenated u8s represented using the hex encoding (each u8 becomes
    /// 2 characters long). The first u8 is used for the VM configuration and
    /// the following 54 represent the vector of helper IDs that should be
    /// during the program execution
    pub fn encode(&self) -> String {
        let mut encoding = format!("{:02x}", self.configuration);

        for helper in &self.allowed_helpers {
            encoding.push_str(&format!("{:02x}", helper));
        }

        encoding
    }

    pub fn decode(data: String) -> Result<VMExecutionRequestMsg, String> {
        let configuration = u8::from_str_radix(&data[0..2], 16).map_err(|e| {
            format!(
                "Unable to parse the vm configuration from the encoded string: {}",
                e
            )
        })?;

        let allowed_helpers = (2..data.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&data[i..i + 2], 16))
            .collect::<Result<Vec<u8>, ParseIntError>>()
            .map_err(|e| format!("Unable to parse: {}", e))?;

        Ok(VMExecutionRequestMsg{ configuration, allowed_helpers })
    }
}

/// Models the request that is sent to the target device to pull a specified
/// binary file from the CoAP fileserver.
/// The handler expects to get a request which consists of the IPv6 address of
/// the machine running the CoAP fileserver and the name of the manifest file
/// specifying which binary needs to be pulled.
#[derive(Serialize, Deserialize, Debug)]
pub struct SuitPullRequest {
    pub ip_addr: String,
    pub manifest: String,
    pub riot_network_interface: String,
}
