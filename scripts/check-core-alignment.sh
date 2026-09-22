#!/usr/bin/env bash
# Finite, no-device validation. Some tests bind loopback TCP/UDP sockets.
set -euo pipefail
cd "$(dirname "$0")/.."
export ROPLAT_SKIP_ASSET_EXPORT=1
export BULLET_SKIP_ASSET_EXPORT=1
cargo metadata --no-deps --format-version 1 --locked >/dev/null
cargo check --workspace --all-targets --locked
cargo check -p robot_behavior --no-default-features --lib --locked
cargo test -p robot_behavior --features roplat --lib --tests --locked
cargo clippy -p robot_behavior --all-features --all-targets --locked -- -D warnings
cargo test -p franka_rust -p libjaka -p libhans --lib --locked
cargo test -p franka_rust --features fci_v8 --lib --locked
cargo test -p roplat_exrobot --test control_flow --locked
cargo test -p libgo2 --features roplat --test roplat_execution --test command_routing --locked
cargo test -p libk1 --test safety --locked
# Opt in only after configuring the Python shared-library search path on this host.
if [[ "${ROPLAT_TEST_MULTILANG:-0}" == "1" ]]; then
    cargo test -p jaka_roplat_multilang --lib --locked
fi
