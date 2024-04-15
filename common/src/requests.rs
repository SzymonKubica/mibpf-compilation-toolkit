use core::num::ParseIntError;

use alloc::{format, string::String, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::{HelperFunctionID, VMConfiguration};

/// Responsible for specifying a request to start executing a given configuration
/// of the VM with access to a specified list of helper functions.
///
/// # Example
/// ```
/// use mibpf_common::{VMConfiguration, TargetVM, BinaryFileLayout, HelperFunctionID};
///
/// // List of helper functions
/// let allowed_helpers = vec![HelperFunctionID::BPF_PRINTF_IDX, HelperFunctionID::BPF_DEBUG_PRINT_IDX]
///
/// // We specify the configuration settings
/// let suit_storage_slot = 0;
/// let binary_layout = BinaryFileLayout::RawObjectFile;
///
/// let configuration = VMConfiguration::new(TargetVM::Rbpf, binary_layout, suit_storage_slot);
///
/// // and construct the request message
/// let request = VMExecutionRequestMsg {
///     configuration,
///     allowed_helpers,
/// };
///
/// // We can now encode it into the compact format before sending to the device
///
/// let encoding = request.encode();
///
/// ```
#[derive(Clone, Debug)]
pub struct VMExecutionRequest {
    /// Encoded representation of the VM config.
    pub configuration: VMConfiguration,
    /// A list of helper functions that the program executing in the VM should be
    /// allowed to call. Each helper function is represented by its index, i.e.
    /// the value that is used as the immediate operand of the CALL instruction
    /// that is used by the VM to call a given helper function. Those indices are
    /// represented by u8 values, which means that currently we can have up to
    /// 256 helper functions.
    pub allowed_helpers: Vec<HelperFunctionID>,
}

impl VMExecutionRequest {
    pub fn new(configuration: VMConfiguration, allowed_helpers: Vec<HelperFunctionID>) -> Self {
        VMExecutionRequest {
            configuration,
            allowed_helpers,
        }
    }

    /// Because of the request entity size constraints when sending the CoAP packets
    /// we need to encode the request into a compact message format to allow for
    /// specifying the highest possible number of helper functions.
    ///
    /// The maximum supported length of the string request payload is 110
    /// (measured empirically - the `aiocoap` documentation doesn't say anything about it)
    ///
    /// Because of this limitation, we encode the request message as a string
    /// of concatenated u8s represented using the hex encoding (each u8 becomes
    /// 2 characters long). The first u8 is used for the VM configuration and
    /// the following 54 represent the vector of helper IDs that should be
    /// during the program execution
    pub fn encode(&self) -> String {
        let mut encoding = format!("{:02x}", self.configuration.encode());

        for helper in &self.allowed_helpers {
            encoding.push_str(&format!("{:02x}", *helper as u8));
        }

        encoding
    }

    pub fn decode(data: String) -> Result<VMExecutionRequest, String> {
        let encoded_configuration = u8::from_str_radix(&data[0..2], 16).map_err(|e| {
            format!(
                "Unable to parse the vm configuration from the encoded string: {}",
                e
            )
        })?;

        let configuration = VMConfiguration::decode(encoded_configuration);

        let allowed_helpers_ids = (2..data.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&data[i..i + 2], 16))
            .collect::<Result<Vec<u8>, ParseIntError>>()
            .map_err(|e| format!("Unable to parse: {}", e))?;

        let allowed_helpers = allowed_helpers_ids
            .into_iter()
            .filter_map(|id| num::FromPrimitive::from_u8(id))
            .collect();

        Ok(VMExecutionRequest {
            configuration,
            allowed_helpers,
        })
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
