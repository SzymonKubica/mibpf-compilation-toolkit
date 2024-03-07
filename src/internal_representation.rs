/*
 * This module contains the structs for the internal representation of objects
 * used by the different subcommands of the tool.
 */

use core::fmt;

use serde::{Serialize, Deserialize};

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


/// Models the request that is sent to the target device to start executing the
/// VM, it specifies the version of the VM that needs to be used to execute it
/// and the index of the location in the SUIT storage from where the program
/// binary needs to be loaded
#[derive(Serialize)]
pub struct ExecuteRequest {
    pub vm_target: VmTarget,
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
