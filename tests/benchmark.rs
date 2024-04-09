mod common;

use common::benchmark_execution;
use internal_representation::BinaryFileLayout;
#[tokio::test]
pub async fn benchmark_binary_layouts() {
    let layouts = vec![
        BinaryFileLayout::FemtoContainersHeader,
        BinaryFileLayout::FunctionRelocationMetadata,
        BinaryFileLayout::RawObjectFile,
    ];

    let environment = common::load_env();

    for layout in layouts {
        println!("Benchmarking {:?}", layout);
        benchmark_execution("bpf_fetch.c", layout, &environment).await;
        benchmark_execution("bpf_fmt_s16_dfp.c", layout, &environment).await;
        benchmark_execution("bpf_fmt_u32_dec.c", layout, &environment).await;
        benchmark_execution("bpf_store.c", layout, &environment).await;
        benchmark_execution("bpf_strlen.c", layout, &environment).await;
        //benchmark_execution("data_relocations.c", layout, &environment).await;
        //benchmark_execution("fletcher32_checksum.c", layout, &environment).await;
        benchmark_execution("gcoap_response_format.c", layout, &environment).await;
        benchmark_execution("inlined_calls.c", layout, &environment).await;
        //benchmark_execution("pc_relative_calls.c", layout, &environment).await;
        benchmark_execution("printf.c", layout, &environment).await;
        println!("Done");
    }

    let layout = BinaryFileLayout::OnlyTextSection;
    println!("Benchmarking {:?}", layout);
    benchmark_execution("bpf_fetch_only_text.c", layout, &environment).await;
    benchmark_execution("bpf_fmt_s16_dfp_only_text.c", layout, &environment).await;
    benchmark_execution("bpf_fmt_u32_dec_only_text.c", layout, &environment).await;
    benchmark_execution("bpf_store_only_text.c", layout, &environment).await;
    benchmark_execution("bpf_strlen_only_text.c", layout, &environment).await;
    benchmark_execution("gcoap_response_format_only_text.c", layout, &environment).await;
    benchmark_execution("inlined_calls_only_text.c", layout, &environment).await;
    benchmark_execution("printf_only_text.c", layout, &environment).await;
    println!("Done");
}
