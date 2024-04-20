mod common;

use mibpf_tools::load_env;
use common::test_jit_execution;
use mibpf_common::BinaryFileLayout;

/// Tests for the simple programs to ensure that the jit compiler works correctly.

#[tokio::test]
async fn jit_add() {
    test_jit("jit_basic-add.c").await;
}

#[tokio::test]
async fn jit_add_reg() {
    test_jit("jit_add-reg.c").await;
}


async fn test_jit(test_program: &str) {
    let env = load_env();
    test_jit_execution(test_program, BinaryFileLayout::OnlyTextSection, &env).await;
}

