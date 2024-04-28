mod common;

use common::test_jit_execution;
use mibpf_common::BinaryFileLayout;
use mibpf_tools::load_env;

/// Tests for the simple programs to ensure that the jit compiler works correctly.

/* Tests for basic arithmetic / logical instructions using immediate operands */
#[tokio::test]
async fn jit_add_immediate() {
    test_jit("jit_add-immediate.c").await;
}
#[tokio::test]
async fn jit_multiply_immediate() {
    test_jit("jit_multiply-immediate.c").await;
}
#[tokio::test]
async fn jit_divide_immediate() {
    test_jit("jit_divide-immediate.c").await;
}
#[tokio::test]
async fn jit_mod_immediate() {
    test_jit("jit_mod-immediate.c").await;
}
#[tokio::test]
async fn jit_subtract_immediate() {
    test_jit("jit_subtract-immediate.c").await;
}
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
async fn jit_divide_reg() {
    test_jit("jit_divide-reg.c").await;
}
#[tokio::test]
async fn jit_mod_reg() {
    test_jit("jit_mod-reg.c").await;
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
#[tokio::test]
async fn jit_asr_reg() {
    test_jit("jit_asr-reg.c").await;
}
#[tokio::test]
async fn jit_lsl_reg() {
    test_jit("jit_lsl-reg.c").await;
}
#[tokio::test]
async fn jit_lsr_reg() {
    test_jit("jit_lsr-reg.c").await;
}
#[tokio::test]
async fn jit_and_reg() {
    test_jit("jit_and-reg.c").await;
}
#[tokio::test]
async fn jit_or_reg() {
    test_jit("jit_or-reg.c").await;
}
#[tokio::test]
async fn jit_xor_reg() {
    test_jit("jit_xor-reg.c").await;
}

/* Tests for load / store operations for various data sizes */
#[tokio::test]
async fn jit_load_store_byte_immediate() {
    test_jit("jit_load-store-byte-immediate.c").await;
}
#[tokio::test]
async fn jit_load_store_halfword_immediate() {
    test_jit("jit_load-store-halfword-immediate.c").await;
}
#[tokio::test]
async fn jit_load_store_word_immediate() {
    test_jit("jit_load-store-word-immediate.c").await;
}

/* Tests for conditional jumps */
// Immediate comparison
#[tokio::test]
async fn jit_jump_eq() {
    test_jit("jit_jump-eq.c").await;
}
#[tokio::test]
async fn jit_jump_ne() {
    test_jit("jit_jump-ne.c").await;
}
#[tokio::test]
async fn jit_jump_ge() {
    test_jit("jit_jump-ge.c").await;
}
#[tokio::test]
async fn jit_jump_gt() {
    test_jit("jit_jump-gt.c").await;
}
#[tokio::test]
async fn jit_jump_le() {
    test_jit("jit_jump-le.c").await;
}
#[tokio::test]
async fn jit_jump_lt() {
    test_jit("jit_jump-lt.c").await;
}
#[tokio::test]
async fn jit_jump_sge() {
    test_jit("jit_jump-sge.c").await;
}
#[tokio::test]
async fn jit_jump_sgt() {
    test_jit("jit_jump-sgt.c").await;
}
#[ignore] // Ignored until ThumbExpandImm in cmp immediate 12 gets fixed
#[tokio::test]
async fn jit_jump_sle() {
    test_jit("jit_jump-sle.c").await;
}
#[tokio::test]
async fn jit_jump_slt() {
    test_jit("jit_jump-slt.c").await;
}
// Immediate comparison, signed versions
#[tokio::test]
async fn jit_jump_ne_reg() {
    test_jit("jit_jump-ne-reg.c").await;
}
// Register comparison: TODO:
//

/* Control flow instructions and proper jump offsets */
#[tokio::test]
async fn jit_for_loop() {
    test_jit("jit_for-loop.c").await;
}
#[tokio::test]
async fn jit_while_loop() {
    test_jit("jit_while-loop.c").await;
}

/* Helper Calls */
#[tokio::test]
async fn jit_helper_call() {
    test_jit("jit_helper-call.c").await;
}

/* Accessing .data and .rodata */
#[tokio::test]
async fn jit_rodata() {
    test_jit("jit_rodata.c").await;
}


async fn test_jit(test_program: &str) {
    let env = load_env();
    test_jit_execution(test_program, BinaryFileLayout::RawObjectFile, &env).await;
}
