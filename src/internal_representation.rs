/*
 * This module contains the structs for the internal representation of objects
 * used by the different subcommands of the tool.
 */

use core::fmt;

use serde::{Deserialize, Serialize};

/// Specifies which version of the eBPF VM is to be used when the program is
/// executed by the microcontroller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
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
            _ => panic!("Invalid VmTarget value"),
        }
    }
}

impl From<&str> for TargetVM {
    fn from(s: &str) -> Self {
        match s {
            "FemtoContainer" => TargetVM::FemtoContainer,
            "rBPF" => TargetVM::Rbpf,
            _ => panic!("Invalid vm target: {}", s),
        }
    }
}

impl fmt::Display for TargetVM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Specifies the different binary file layouts that are supported by the VMs
/// Note that only the FemtoContainersHeader layout is compatible with the
/// FemtoContainer VM.
#[derive(Serialize, Eq, Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
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
    /// TODO: figure out if it is even feasible to perform that on the embedded
    /// device.
    RawObjectFile = 3,
}

impl From<&str> for BinaryFileLayout {
    fn from(s: &str) -> Self {
        match s {
            "OnlyTextSection" => BinaryFileLayout::OnlyTextSection,
            "FemtoContainersHeader" => BinaryFileLayout::FemtoContainersHeader,
            "FunctionRelocationMetadata" => BinaryFileLayout::FunctionRelocationMetadata,
            "RawObjectFile" => BinaryFileLayout::RawObjectFile,
            _ => panic!("Invalid binary layout: {}", s),
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

#[derive(Clone, Serialize, Deserialize)]
pub struct VMExecutionRequestMsg {
    pub configuration: u8,
    pub available_helpers: [u8; 3],
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
