use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmTarget {
    FemtoContainers,
    RBPF,
}

impl From<String> for VmTarget {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Femto-Containers" => VmTarget::FemtoContainers,
            "rBPF" => VmTarget::RBPF,
            _ => panic!("Invalid vm target: {}", s),
        }
    }
}

impl fmt::Display for VmTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn handle_compile(bpf_source_file: &str, target: VmTarget, output_file: &str) {
    match target {
        VmTarget::FemtoContainers => compile_fc(bpf_source_file, output_file),
        VmTarget::RBPF => compile_rbpf(bpf_source_file, output_file)
    }
}

fn compile_rbpf(bpf_source_file: &str, output_file: &str) {
    todo!()
}

fn compile_fc(bpf_source_file: &str, output_file: &str) {
    todo!()
}
