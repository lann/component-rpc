#!/usr/bin/env bash

set -e

cargo component -V > /dev/null || (echo 'Install cargo-component!' && exit 1)

(cd examples/hello-rpc && cargo component build --release)

cargo run examples/hello-rpc/target/wasm32-unknown-unknown/release/hello_rpc.wasm
