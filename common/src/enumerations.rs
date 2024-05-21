/// This module defines all of the structs and enums that are shared between
/// the different components of the project (cli tool, mibpf-server). The idea
/// is that the internal representation of all structures in the system can be
/// easily imported into each one of the components as a library.
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
    /// The SUIT storage slot from where the program should be loaded.
    pub suit_slot: usize,
    /// Defines the order of information present in the loaded program. It is
    /// needed by the VM to correctly find the first instruction in the program
    /// and extract the metadata.
    pub binary_layout: BinaryFileLayout,
    /// Defines at what point of the pipeline the verification of accesses to
    /// helper function calls should take place.
    pub helper_access_verification: HelperAccessVerification,
    /// Informs the VM from where the list of available helpers should be sourced
    pub helper_access_list_source: HelperAccessListSource,
    /// Controlls whether the VM should use the JIT-compiled eBPF programs.
    pub jit: bool,
    /// Controlls whether the jitted program is to be compiled preflight
    /// or loaded from jit program storage.
    pub jit_compile: bool,
}

impl VMConfiguration {
    pub fn new(
        vm_target: TargetVM,
        suit_slot: usize,
        binary_layout: BinaryFileLayout,
        helper_access_verification: HelperAccessVerification,
        helper_access_list_source: HelperAccessListSource,
        jit: bool,
        jit_compile: bool,
    ) -> Self {
        VMConfiguration {
            vm_target,
            binary_layout,
            suit_slot,
            helper_access_verification,
            helper_access_list_source,
            jit,
            jit_compile,
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
    /// - bit 0: the least significant bit specifies whether we should use the rbpf
    ///   or the FemtoContainers VM. 0 corresponds to rbpf and 1 to FemtoContainers.
    /// - bits 1-4: the next four bits specify the SUIT storage slot storing the eBPF program
    ///   bytecode (up to 16 available program slots).
    /// - bits 5-6: the next two bits specify the binary file layout that the VM should
    ///   expect in the loaded program
    /// - bits 7-8: the next two bits specify the time in the pipeline at which the verification
    ///   of accesses to helper functions should take place.
    /// - bit 9: the next bit specifies whether the list
    ///   of allowed helper functions should be parsed from the program binary
    ///   (only supported for the [`BinaryFileLayout::ExtendedHeader`]) or taken
    ///   from the execution request payload.
    /// - bit 10: The next bit specifies whether we should use jit-compiled programs.
    /// - bit 11: The next bit specifies if we should run the jit-compilation or
    ///   use one of the pre-compiled programs that are present in the jit storage.
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

    pub fn encode(&self) -> u16 {
        let mut encoding: u16 = 0;
        encoding |= self.vm_target as u16 & 0b1;
        encoding |= (self.suit_slot as u16 & 0b1111) << 1;
        encoding |= (self.binary_layout as u16 & 0b11) << 5;
        encoding |= (self.helper_access_verification as u16 & 0b11) << 7;
        encoding |= (self.helper_access_list_source as u16 & 0b1) << 9;
        encoding |= (self.jit as u16 & 0b1) << 10;
        encoding |= (self.jit_compile as u16 & 0b1) << 11;
        encoding
    }

    /// Decodes the VM configuration according to the encoding specified above.
    pub fn decode(encoding: u16) -> Self {
        VMConfiguration {
            vm_target: TargetVM::from((encoding & 0b1) as u8),
            suit_slot: ((encoding >> 1) & 0b1111) as usize,
            binary_layout: BinaryFileLayout::from(((encoding >> 5) & 0b11) as u8),
            helper_access_verification: HelperAccessVerification::from(
                ((encoding >> 7) & 0b11) as u8,
            ),
            helper_access_list_source: HelperAccessListSource::from(((encoding >> 9) & 0b1) as u8),
            jit: ((encoding >> 10) & 0b1) == 1,
            jit_compile: ((encoding >> 11) & 0b1) == 1,
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
            _ => Err(format!("Unknown target VM: {}", s)),
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
    ExtendedHeader = 2,
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
            "ExtendedHeader" => Ok(BinaryFileLayout::ExtendedHeader),
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
            2 => BinaryFileLayout::ExtendedHeader,
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

/// Controlls at what point of the pipeline the verification of calls to helper
/// functions happens.
#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Copy, Clone)]
pub enum HelperAccessVerification {
    /// The program bytecode is parsed immediately after compilation, before it
    /// gets signed and sent to the target device.
    AheadOfTime = 0,
    /// The program is verified when it gets deployed to the target microcontroller,
    /// the SUIT storage worker fetches the program, then we perform verification
    /// on it and write it into the SUIT storage if it is well behaved. From there
    /// subsequent execution requests can take the program and execute it.
    LoadTime = 1,
    /// The verification happens immediately before the VM starts executing the
    /// program.
    PreFlight = 2,
    /// The VM performs runtime checks before making calls to helpers.
    Runtime = 3,
}

impl FromStr for HelperAccessVerification {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AheadOfTime" => Ok(HelperAccessVerification::AheadOfTime),
            "LoadTime" => Ok(HelperAccessVerification::LoadTime),
            "PreFlight" => Ok(HelperAccessVerification::PreFlight),
            "Runtime" => Ok(HelperAccessVerification::Runtime),
            _ => Err(format!("Unknown helper access verification type: {}", s)),
        }
    }
}

impl From<u8> for HelperAccessVerification {
    fn from(val: u8) -> Self {
        match val {
            0 => HelperAccessVerification::AheadOfTime,
            1 => HelperAccessVerification::LoadTime,
            2 => HelperAccessVerification::PreFlight,
            3 => HelperAccessVerification::Runtime,
            _ => panic!("Unknown helper access verification type: {}", val),
        }
    }
}

/// Specifies from where we should take the list of allowed helper functions
/// when verifying the program.
#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Copy, Clone)]
pub enum HelperAccessListSource {
    /// The list of allowed helpers is appended to the payload of the request
    /// to start executing the VM.
    ExecuteRequest = 0,
    /// The list of allowed helpers is embedded in the program bytecode. Note
    /// that currently this is only supported with the [`BinaryFileLayout::FunctionRelocationMetadata`]
    BinaryMetadata = 1,
}

impl FromStr for HelperAccessListSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ExecuteRequest" => Ok(HelperAccessListSource::ExecuteRequest),
            "BinaryMetadata" => Ok(HelperAccessListSource::BinaryMetadata),
            _ => Err(format!("Unknown helper access list source: {}", s)),
        }
    }
}

impl From<u8> for HelperAccessListSource {
    fn from(val: u8) -> Self {
        match val {
            0 => HelperAccessListSource::ExecuteRequest,
            1 => HelperAccessListSource::BinaryMetadata,
            _ => panic!("Unknown helper access list source: {}", val),
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
            1,
            BinaryFileLayout::FemtoContainersHeader,
            HelperAccessVerification::PreFlight,
            HelperAccessListSource::BinaryMetadata,
            true,
            false,
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
    BPF_SAUL_REG_READ_TEMP = 0x34,

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
    BPF_KEYPAD_GET_INPUT = 0x84,
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
