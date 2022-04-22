#!/bin/bash
set -e
cargo build
rm -rf examples/intra/target
cd examples/intra && RUSTFLAGS="--emit=mir" RUSTC="/Users/xsh/code/deadlock-lsp/target/debug/rustc" RUST_LOG=trace RUST_BACKTRACE=full __DL_OUT="/Users/xsh/code/deadlock-lsp/a.json" cargo check