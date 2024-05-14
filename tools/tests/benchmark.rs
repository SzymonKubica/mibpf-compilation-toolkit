mod common;

use common::benchmark_execution;
use mibpf_common::{BinaryFileLayout, TargetVM};

use crate::common::{benchmark_fletcher_16, benchmark_fletcher_16_native, benchmark_jit_execution};
#[ignore]
#[tokio::test]
pub async fn benchmark_binary_layouts() {
    let layouts = vec![
        BinaryFileLayout::RawObjectFile,
        BinaryFileLayout::FemtoContainersHeader,
        BinaryFileLayout::ExtendedHeader,
    ];

    let environment = mibpf_tools::load_env();

    let target = TargetVM::Rbpf;
    for layout in layouts {
        println!("Benchmarking {:?}", layout);
        benchmark_execution("bpf_fetch.c", layout, &environment, target).await;
        benchmark_execution("bpf_fmt_s16_dfp.c", layout, &environment, target).await;
        benchmark_execution("bpf_fmt_u32_dec.c", layout, &environment, target).await;
        benchmark_execution("bpf_store.c", layout, &environment, target).await;
        benchmark_execution("bpf_strlen.c", layout, &environment, target).await;
        //benchmark_execution("data_relocations.c", layout, &environment).await;
        //benchmark_execution("fletcher32_checksum.c", layout, &environment).await;
        benchmark_execution("gcoap_response_format.c", layout, &environment, target).await;
        benchmark_execution("inlined_calls.c", layout, &environment, target).await;
        //benchmark_execution("pc_relative_calls.c", layout, &environment).await;
        benchmark_execution("printf.c", layout, &environment, target).await;
        benchmark_execution(
            "jit_fletcher16_checksum_320B_data.c",
            layout,
            &environment,
            target,
        )
        .await;
        println!("Done");
    }

    let layout = BinaryFileLayout::OnlyTextSection;
    println!("Benchmarking {:?}", layout);
    benchmark_execution("bpf_fetch_only_text.c", layout, &environment, target).await;
    benchmark_execution("bpf_fmt_s16_dfp_only_text.c", layout, &environment, target).await;
    benchmark_execution("bpf_fmt_u32_dec_only_text.c", layout, &environment, target).await;
    benchmark_execution("bpf_store_only_text.c", layout, &environment, target).await;
    benchmark_execution("bpf_strlen_only_text.c", layout, &environment, target).await;
    benchmark_execution(
        "gcoap_response_format_only_text.c",
        layout,
        &environment,
        target,
    )
    .await;
    benchmark_execution("inlined_calls_only_text.c", layout, &environment, target).await;
    benchmark_execution("printf_only_text.c", layout, &environment, target).await;
    benchmark_execution(
        "jit_fletcher16_checksum_320B_data.c",
        layout,
        &environment,
        target,
    )
    .await;
    println!("Done");
}

#[ignore]
#[tokio::test]
pub async fn benchmark_femtocontainers() {
    // We also benchmark native femto-containers.
    let target = TargetVM::FemtoContainer;
    let layout = BinaryFileLayout::FemtoContainersHeader;
    let environment = mibpf_tools::load_env();
    benchmark_execution("bpf_fetch.c", layout, &environment, target).await;
    benchmark_execution("bpf_fmt_s16_dfp.c", layout, &environment, target).await;
    benchmark_execution("bpf_fmt_u32_dec.c", layout, &environment, target).await;
    benchmark_execution("bpf_store.c", layout, &environment, target).await;
    benchmark_execution("bpf_strlen.c", layout, &environment, target).await;
    //benchmark_execution("data_relocations.c", layout, &environment).await;
    //benchmark_execution("fletcher32_checksum.c", layout, &environment).await;
    benchmark_execution("gcoap_response_format.c", layout, &environment, target).await;
    benchmark_execution("inlined_calls.c", layout, &environment, target).await;
    //benchmark_execution("pc_relative_calls.c", layout, &environment).await;
    benchmark_execution(
        "jit_fletcher16_checksum_320B_data.c",
        layout,
        &environment,
        target,
    )
    .await;
    println!("Done");
}

#[ignore]
#[tokio::test]
pub async fn benchmark_jit_examples() {
    // We also benchmark native femto-containers.
    let environment = mibpf_tools::load_env();
    benchmark_jit_execution("bpf_fetch.c", &environment).await;
    benchmark_jit_execution("bpf_fmt_s16_dfp.c", &environment).await;
    benchmark_jit_execution("bpf_fmt_u32_dec.c", &environment).await;
    benchmark_jit_execution("bpf_store.c", &environment).await;
    benchmark_jit_execution("bpf_strlen.c", &environment).await;
    //benchmark_execution("data_relocations.c", layout, &environment).await;
    //benchmark_execution("fletcher32_checksum.c", layout, &environment).await;
    //benchmark_execution("gcoap_response_format.c", layout, &environment).await;
    // INNLINED calls aren't supported by femtocontainers as they don't support
    // the return address stack.
    benchmark_jit_execution("inlined_calls.c", &environment).await;
    //benchmark_execution("pc_relative_calls.c", layout, &environment).await;
    benchmark_jit_execution("jit_fletcher16_checksum_320B_data.c", &environment).await;
    println!("Done");
}

#[ignore]
#[tokio::test]
pub async fn benchmark_native() {
    let environment = mibpf_tools::load_env();
    for i in 1..=6 {
        benchmark_fletcher_16_native(i, &environment).await;
    }
}

#[ignore]
#[tokio::test]
pub async fn benchmark_fc_interpreter() {
    let environment = mibpf_tools::load_env();
    for data_size in 1..=6 {
        benchmark_fletcher_16(
            data_size,
            &environment,
            BinaryFileLayout::FemtoContainersHeader,
            false,
        )
        .await;
    }
}

#[ignore]
#[tokio::test]
pub async fn benchmark_extended_interpreter() {
    let environment = mibpf_tools::load_env();
    for data_size in 1..=6 {
        benchmark_fletcher_16(
            data_size,
            &environment,
            BinaryFileLayout::ExtendedHeader,
            false,
        )
        .await;
    }
}

#[ignore]
#[tokio::test]
pub async fn benchmark_raw_object_file_interpreter() {
    let environment = mibpf_tools::load_env();
    for data_size in 1..=6 {
        benchmark_fletcher_16(
            data_size,
            &environment,
            BinaryFileLayout::RawObjectFile,
            false,
        )
        .await;
    }
}

#[ignore]
#[tokio::test]
pub async fn benchmark_jit() {
    let environment = mibpf_tools::load_env();
    for data_size in 1..=6 {
        benchmark_fletcher_16(
            data_size,
            &environment,
            BinaryFileLayout::RawObjectFile,
            true,
        )
        .await;
    }
}
