mod common;

use std::{collections::HashMap, env};

use common::benchmark_execution;
use micro_bpf_common::{BinaryFileLayout, TargetVM};

use crate::common::{
    benchmark_fletcher_16, benchmark_fletcher_16_native, benchmark_jit_execution, BenchmarkResponse, benchmark_memory_access_checks,
};

const BENCHMARK_SOURCES: [&'static str; 10] = [
    "bpf_fetch.c",
    "bpf_fmt_s16_dfp.c",
    "bpf_fmt_u32_dec.c",
    "bpf_store.c",
    "bpf_strlen.c",
    "inlined_calls.c",
    "printf.c",
    "jit_fletcher16_checksum_320B_data.c",
    "sensor-processing.c",
    "sensor-processing-from-storage.c",
];

const BENCHMARK_SOURCES_FOR_ONLY_TEXT_SECTION_LAYOUT: [&'static str; 10] = [
    "bpf_fetch_only_text.c",
    "bpf_fmt_s16_dfp_only_text.c",
    "bpf_fmt_u32_dec_only_text.c",
    "bpf_store_only_text.c",
    "bpf_strlen_only_text.c",
    "inlined_calls_only_text.c",
    "printf_only_text.c",
    "jit_fletcher16_checksum_320B_data.c",
    "sensor-processing.c",
    "sensor-processing-from-storage.c",
];

#[ignore]
#[tokio::test]
pub async fn benchmark_only_text_section_layout() {
    let layout = BinaryFileLayout::OnlyTextSection;
    let environment = micro_bpf_tools::load_env();
    let target = TargetVM::Rbpf;

    let mut results: HashMap<&'static str, BenchmarkResponse> = HashMap::new();
    for source in BENCHMARK_SOURCES_FOR_ONLY_TEXT_SECTION_LAYOUT.iter() {
        let response = benchmark_execution(*source, layout, &environment, target).await;
        results.insert(source, response);
    }
    save_results("only-text-section-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_extended_header() {
    let results = benchmark_layout(BinaryFileLayout::ExtendedHeader, TargetVM::Rbpf).await;
    save_results("extended-header-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_femtocontainers_header() {
    let results = benchmark_layout(BinaryFileLayout::FemtoContainersHeader, TargetVM::Rbpf).await;
    save_results("femtocontainers-header-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_raw_object_file() {
    let results = benchmark_layout(BinaryFileLayout::RawObjectFile, TargetVM::Rbpf).await;
    save_results("raw-object-file-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_femtocontainers() {
    let results = benchmark_layout(
        BinaryFileLayout::FemtoContainersHeader,
        TargetVM::FemtoContainer,
    )
    .await;
    save_results("femtocontainers-results.json", results);
}

pub fn save_results<T: serde::Serialize>(file_name: &str, results: T) {
    let save_results = env::var("SAVE_RESULTS").unwrap_or_else(|_| "False".to_string());
    match save_results.as_str() {
        "False" => return,
        _ => (),
    }
    let _ = std::fs::write(file_name, serde_json::to_string_pretty(&results).unwrap());
}

pub async fn benchmark_layout(
    layout: BinaryFileLayout,
    target: TargetVM,
) -> HashMap<&'static str, BenchmarkResponse> {
    let environment = micro_bpf_tools::load_env();
    let mut results: HashMap<&'static str, BenchmarkResponse> = HashMap::new();
    for source in BENCHMARK_SOURCES.iter() {
        let response = benchmark_execution(*source, layout, &environment, target).await;
        results.insert(source, response);
    }
    results
}

#[ignore]
#[tokio::test]
pub async fn benchmark_jit() {
    let environment = micro_bpf_tools::load_env();
    let mut results: HashMap<&'static str, BenchmarkResponse> = HashMap::new();
    for source in BENCHMARK_SOURCES.iter() {
        let response = benchmark_jit_execution(*source, &environment).await;
        results.insert(source, response);
    }
    save_results("jit-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_fletcher_native() {
    let environment = micro_bpf_tools::load_env();
    let mut results = HashMap::new();
    for data_size in 1..=6 {
        let response = benchmark_fletcher_16_native(data_size, &environment).await;
        let base: u32 = 2;
        let bytes = 80 * base.pow((data_size - 1) as u32);
        results.insert(bytes, response);
    }
    save_results("native-fletcher-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_fletcher_femtocontainers_header() {
    let results = benchmark_fletcher(BinaryFileLayout::FemtoContainersHeader).await;
    println!(
        "Benchmark results: {}",
        serde_json::to_string_pretty(&results).unwrap()
    );
    save_results("femtocontainers-header-fletcher-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_fletcher_extended_header() {
    let results = benchmark_fletcher(BinaryFileLayout::ExtendedHeader).await;
    println!(
        "Benchmark results: {}",
        serde_json::to_string_pretty(&results).unwrap()
    );
    save_results("extended-header-fletcher-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_fletcher_raw_object_file() {
    let results = benchmark_fletcher(BinaryFileLayout::RawObjectFile).await;
    println!(
        "Benchmark results: {}",
        serde_json::to_string_pretty(&results).unwrap()
    );
    save_results("raw-object-file-fletcher-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn benchmark_fletcher_jit() {
    let environment = micro_bpf_tools::load_env();
    let mut results = HashMap::new();
    for data_size in 1..=6 {
        let response = benchmark_fletcher_16(
            data_size,
            &environment,
            BinaryFileLayout::RawObjectFile,
            true,
        )
        .await;
        let base: u32 = 2;
        let bytes = 80 * base.pow((data_size - 1) as u32);
        results.insert(bytes, response);
    }
    save_results("jit-fletcher-results.json", results);
}

#[ignore]
#[tokio::test]
pub async fn bench_memory_access_checks() {
    let environment = micro_bpf_tools::load_env();
    let results = benchmark_memory_access_checks(&environment).await;
    save_results("memory-access-checks.json", results);
}

pub async fn benchmark_fletcher(layout: BinaryFileLayout) -> HashMap<u32, BenchmarkResponse> {
    let environment = micro_bpf_tools::load_env();
    let mut results = HashMap::new();
    for data_size in 1..=6 {
        let response = benchmark_fletcher_16(data_size, &environment, layout, false).await;
        let base: u32 = 2;
        let bytes = 80 * base.pow((data_size - 1) as u32);
        results.insert(bytes, response);
    }
    results
}
