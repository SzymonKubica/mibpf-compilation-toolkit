/*
 * This module contains the structs for the internal representation of objects
 * used by the different subcommands of the tool.
 */

use core::fmt;

use serde::{Deserialize, Serialize};

/// Specifies which version of the eBPF VM is to be used when the program is
/// executed by the microcontroller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum VmTarget {
    /// The eBPF program will be executed by the FemtoContainer VM.
    FemtoContainer,
    /// The eBPF program will be executed by the rBPF VM.
    Rbpf,
}

impl From<&str> for VmTarget {
    fn from(s: &str) -> Self {
        match s {
            "FemtoContainer" => VmTarget::FemtoContainer,
            "rBPF" => VmTarget::Rbpf,
            _ => panic!("Invalid vm target: {}", s),
        }
    }
}

impl fmt::Display for VmTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Specifies the different binary file layouts that are supported by the VMs
/// Note that only the FemtoContainersHeader layout is compatible with the
/// FemtoContainer VM.
#[derive(Serialize)]
pub enum BinaryFileLayout {
    /// The most basic layout of the produced binary. Used by the original version
    /// of the rBPF VM. It only includes the .text section from the ELF file.
    /// The limitation is that none of the .rodata relocations work in this case.
    OnlyTextSection,
    /// A custom layout used by the VM version implemented for Femto-Containers.
    /// It starts with a header section which specifies lengths of remaining sections
    /// (.data, .rodata, .text). See [`crate::relocate::Header`] for more detailed
    /// description of the header format.
    FemtoContainersHeader,
    /// An extension of the [`BytecodeLayout::FemtoContainersHeader`] bytecode
    /// layout. It appends additional metadata used for resolving function
    /// relocations and is supported by the modified version of rBPF VM.
    FunctionRelocationMetadata,
    /// Raw object files are sent to the device and the relocations are performed
    /// there. This allows for maximum compatibility (e.g. .data relocations)
    /// however it comes with a burden of an increased memory requirements.
    /// TODO: figure out if it is even feasible to perform that on the embedded
    /// device.
    RawObjectFile,
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

/// Models the request that is sent to the target device to start executing the VM, it specifies
/// the version of the VM that needs to be used to execute it, the layout of the bytecode file that
/// the VM should expect and the index of the location in the SUIT storage from where the program
/// binary needs to be loaded
#[derive(Serialize)]
pub struct ExecuteRequest {
    pub vm_target: VmTarget,
    pub binary_layout: BinaryFileLayout,
    pub suit_location: usize,
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
}
