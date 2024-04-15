/*
 * This module contains the structs for the internal representation of objects
 * used by the different subcommands of the tool.
 */

use core::fmt;
use core::str::FromStr;

use alloc::{format, string::String};
use enum_iterator::Sequence;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

/// Configures a particular instance of the eBPF VM, it specifies the target version
/// of the VM implementation, the binary file layout that the VM should expect
/// in the loaded bytecode and the SUIT storage slot from where the program
/// should be loaded.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VMConfiguration {
    /// The version of the VM implementation that will be used by the VM instance.
    pub vm_target: TargetVM,
    /// Defines the order of information present in the loaded program. It is
    /// needed by the VM to correctly find the first instruction in the program
    /// and extract the metadata.
    pub binary_layout: BinaryFileLayout,
    /// The SUIT storage slot from where the program should be loaded.
    pub suit_slot: usize,
}

impl VMConfiguration {
    pub fn new(vm_target: TargetVM, binary_layout: BinaryFileLayout, suit_slot: usize) -> Self {
        VMConfiguration {
            vm_target,
            binary_layout,
            suit_slot,
        }
    }

    /// Encodes the VM configuration into a u8. The reason we need this is that
    /// RIOT message passing IPC infrastructure limits the size of the transported
    /// messages to 32 bits. In order to fully specify a given VM execution,
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
    ///
    /// # Example
    /// ```
    /// // Initialize the configuration object.
    ///
    /// use mibpf_common::{TargetVM, BinaryFileLayout, VMConfiguration};
    /// let config = VMConfiguration::new(TargetVM::FemtoContainer, BinaryFileLayout::FemtoContainersHeader, 0);
    ///
    /// // Encode the configuration.
    /// let encoding = config.encode();
    /// // The encoded value represented as a bit string will be 0b001001
    /// //                                                           ^l^s
    /// // Where l above points to the the set bit corresponding to 2 which
    /// // is the value of the FemtoContainersHeader variant of the enum and
    /// // s points to the suit_slot field of the configuation
    /// ```

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
            suit_slot: ((encoding >> 1) & 0b1) as usize,
            binary_layout: BinaryFileLayout::from((encoding >> 2) & 0b11),
        }
    }
}

/// The target implementation of the VM used to run the program.
/// The reason we need this is that we want to compare the rbpf VM implementaion
/// against the baseline implementation of the Femto-Containers VM.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetVM {
    /// The eBPF program will be executed by the rBPF VM.
    Rbpf = 0,
    /// The eBPF program will be executed by the FemtoContainer VM.
    FemtoContainer = 1,
}

impl From<u8> for TargetVM {
    fn from(v: u8) -> Self {
        match v {
            0 => TargetVM::Rbpf,
            1 => TargetVM::FemtoContainer,
            _ => panic!("Invalid TargetVM enum index value: {}", v),
        }
    }
}

impl FromStr for TargetVM {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rBPF" => Ok(TargetVM::Rbpf),
            "FemtoContainer" => Ok(TargetVM::FemtoContainer),
            _ => Err(format!("Unknown target VM: {}", s))
        }
    }
}

impl fmt::Display for TargetVM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Specifies the binary file layouts that are supported by the VMs. In this context
/// the binary layout refers to the structure of the program that is loaded and
/// then executed by the VM. A simple example of a layout is where the program
/// consists of only the `.text` section that was stripped from the ELF file
/// (this approach was originally used by rbpf implementation of the VM).
///
/// Note:
/// FemtoContainer VM is only compatible with the FemtoContainersHeader binary layout.
#[repr(u8)]
#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Copy, Clone)]
pub enum BinaryFileLayout {
    /// The most basic layout of the produced binary. Used by the original version
    /// of the rBPF VM. It only includes the .text section from the ELF file.
    /// The limitation is that none of the .rodata relocations work in this case.
    OnlyTextSection = 0,
    /// A custom layout used by the VM version implemented for Femto-Containers.
    /// It starts with a header section which specifies lengths of remaining sections
    /// (.data, .rodata, .text). See [`crate::relocate::Header`] for more detailed
    /// description of the header format.
    FemtoContainersHeader = 1,
    /// An extension of the [`BytecodeLayout::FemtoContainersHeader`] bytecode
    /// layout. It appends additional metadata used for resolving function
    /// relocations and is supported by the modified version of rBPF VM.
    FunctionRelocationMetadata = 2,
    /// Raw object files are sent to the device and the relocations are performed
    /// there. This allows for maximum compatibility (e.g. .data relocations)
    /// however it comes with a burden of an increased memory requirements.
    RawObjectFile = 3,
}

impl FromStr for BinaryFileLayout {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OnlyTextSection" => Ok(BinaryFileLayout::OnlyTextSection),
            "FemtoContainersHeader" => Ok(BinaryFileLayout::FemtoContainersHeader),
            "FunctionRelocationMetadata" => Ok(BinaryFileLayout::FunctionRelocationMetadata),
            "RawObjectFile" => Ok(BinaryFileLayout::RawObjectFile),
            _ => Err(format!("Unknown binary file layout: {}", s)),
        }
    }
}

impl From<u8> for BinaryFileLayout {
    fn from(val: u8) -> Self {
        match val {
            0 => BinaryFileLayout::OnlyTextSection,
            1 => BinaryFileLayout::FemtoContainersHeader,
            2 => BinaryFileLayout::FunctionRelocationMetadata,
            3 => BinaryFileLayout::RawObjectFile,
            _ => panic!("Unknown binary file layout: {}", val),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionModel {
  /// The VM instance is spawned in the thread that is handling the network
  /// request to execute the VM, the programs running using this model should be
  /// short lived and terminate quickly enough so that the response can be sent
  /// back to the client (this response usually contains the return value of the
  /// program)
  ShortLived,
  /// Similar to the ShortLived execution model, but in this case the program has
  /// access to the packet data and can write the response there using helpers.
  /// The program can format the CoAP response accordingly and so it allows for
  /// specifying custom responses.
  WithAccessToCoapPacket,
  /// The VM instances are spawned on a separate thread (by communicating a request
  /// to start executing using message passing IPC provided by RIOT). The VM
  /// can then run as long as needed and there is no way of early terminating
  /// its execution
  LongRunning,
  /// Similar as ShortLived but more data is collected when the vm runs.
  Benchmark,
}

impl FromStr for ExecutionModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ShortLived" => Ok(ExecutionModel::ShortLived),
            "WithAccessToCoapPacket" => Ok(ExecutionModel::WithAccessToCoapPacket),
            "LongRunning" => Ok(ExecutionModel::LongRunning),
            _ => Err(format!("Unknown execution model: {}", s)),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_after_encode_is_identity() {
        let configuration = VMConfiguration::new(
            TargetVM::FemtoContainer,
            BinaryFileLayout::FemtoContainersHeader,
            1,
        );

        let encoded = configuration.encode();
        let decoded = VMConfiguration::decode(encoded);

        assert_eq!(configuration, decoded);
    }
}

/// This enum defines all available helper IDs. The requirement is that every
/// single helper function ID is unique, hence we store them in an enum.
/// Files containing helper definitions should depend on enumeration variants
/// defined in this file.
///
/// In case of the helper functions that were implemented for the FemtoContainer
/// VM, we use the same set of IDs for compatibility.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Sequence, FromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
pub enum HelperFunctionID {
    /* Print/debug helper functions */
    BPF_PRINTF_IDX = 0x01,
    BPF_DEBUG_PRINT_IDX = 0x03,

    /* Memory copy helper functions */
    BPF_MEMCPY_IDX = 0x02,

    /* Key/value store functions */
    BPF_STORE_LOCAL_IDX = 0x10,
    BPF_STORE_GLOBAL_IDX = 0x11,
    BPF_FETCH_LOCAL_IDX = 0x12,
    BPF_FETCH_GLOBAL_IDX = 0x13,

    /* Saul functions */
    BPF_SAUL_REG_FIND_NTH_IDX = 0x30,
    BPF_SAUL_REG_FIND_TYPE_IDX = 0x31,
    BPF_SAUL_REG_READ_IDX = 0x32,
    BPF_SAUL_REG_WRITE_IDX = 0x33,

    /* (g)coap functions */
    BPF_GCOAP_RESP_INIT_IDX = 0x40,
    BPF_COAP_OPT_FINISH_IDX = 0x41,
    BPF_COAP_ADD_FORMAT_IDX = 0x42,
    BPF_COAP_GET_PDU_IDX = 0x43,

    /* Format and string functions */
    BPF_STRLEN_IDX = 0x52,
    BPF_FMT_S16_DFP_IDX = 0x50,
    BPF_FMT_U32_DEC_IDX = 0x51,

    /* Time(r) functions */
    BPF_NOW_MS_IDX = 0x20,

    /* ZTIMER */
    BPF_ZTIMER_NOW_IDX = 0x60,
    BPF_PERIODIC_WAKEUP_IDX = 0x61,

    BPF_GPIO_READ_INPUT = 0x70,
    BPF_GPIO_READ_RAW = 0x71,
    BPF_GPIO_WRITE = 0x72,

    /* HD44780 LCD */
    BPF_HD44780_INIT = 0x80,
    BPF_HD44780_CLEAR = 0x81,
    BPF_HD44780_PRINT = 0x82,
    BPF_HD44780_SET_CURSOR = 0x83,
}

impl Into<u32> for HelperFunctionID {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Into<u8> for HelperFunctionID {
    fn into(self) -> u8 {
        self as u8
    }
}
