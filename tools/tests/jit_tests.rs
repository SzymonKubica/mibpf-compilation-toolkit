mod common;

use mibpf_tools::load_env;
use common::test_jit_execution;
use mibpf_common::BinaryFileLayout;

/// Tests for the simple programs to ensure that the jit compiler works correctly.

/* Tests for basic arithmetic / logical instructions using immediate operands */
#[tokio::test]
async fn jit_add_immediate() {
    test_jit("jit_add-immediate.c").await;
}

#[tokio::test]
async fn jit_subtract_immediate() {
    test_jit("jit_subtract-immediate.c").await;
}

#[tokio::test]
async fn jit_multiply_immediate() {
    test_jit("jit_multiply-immediate.c").await;
}


#[ignore] // Ignored until we have better support for negative numbers.
#[tokio::test]
async fn jit_asr() {
    test_jit("jit_asr-immediate.c").await;
}

#[tokio::test]
async fn jit_lsl_immediate() {
    test_jit("jit_lsl-immediate.c").await;
}

#[tokio::test]
async fn jit_lsr_immediate() {
    test_jit("jit_lsr-immediate.c").await;
}

#[tokio::test]
async fn jit_and_immediate() {
    test_jit("jit_and-immediate.c").await;
}
#[tokio::test]
async fn jit_or_immediate() {
    test_jit("jit_or-immediate.c").await;
}
#[tokio::test]
async fn jit_xor_immediate() {
    test_jit("jit_xor-immediate.c").await;
}



// TODO:
// divide (later)
// mod (later)
// lsl
// lsr
// asr
// and
// or
// xor

/* Tests for basic arithmetic / logical instructions operating on registers */
#[tokio::test]
async fn jit_add_reg() {
    test_jit("jit_add-reg.c").await;
}

#[tokio::test]
async fn jit_multiply_reg() {
    test_jit("jit_multiply-reg.c").await;
}

#[tokio::test]
async fn jit_subtract_reg() {
    test_jit("jit_subtract-reg.c").await;
}

#[ignore]
#[tokio::test]
async fn jit_fletcher() {
    test_jit("jit_fletcher32_checksum.c").await;
}




async fn test_jit(test_program: &str) {
    let env = load_env();
    test_jit_execution(test_program, BinaryFileLayout::OnlyTextSection, &env).await;
}

