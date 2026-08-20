#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

cargo build -p wasmtime-lib --target aarch64-unknown-none
mkdir -p target
wat2wasm wasmtime-lib/sample/sample.wat -o target/wasmtime-lib-sample.wasm
"${CC_AARCH64:-aarch64-unknown-linux-gnu-gcc}" \
  wasmtime-lib/sample/sample.c \
  target/aarch64-unknown-none/debug/libwasmtime_lib.a \
  -o target/wasmtime-lib-sample
"${QEMU_AARCH64:-qemu-aarch64}" \
  target/wasmtime-lib-sample \
  target/wasmtime-lib-sample.wasm
