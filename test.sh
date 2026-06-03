#!/bin/bash
set -euxo pipefail

repo_dir=$(cd "$(dirname "$0")" && pwd)

# Validate POSIX tools on the host and on the xv8 target.
cargo test --release --manifest-path "$repo_dir/posix/Cargo.toml"
cargo build --release --manifest-path "$repo_dir/posix/Cargo.toml" --target riscv64gc-unknown-none-elf --no-default-features

# Validate xv8 std/runtime support for the xv8 target.
cargo build --release --manifest-path "$repo_dir/xv8rust/Cargo.toml" --target riscv64gc-unknown-none-elf

# Run the xv8 integration test harness.
(cd "$repo_dir/xv8" && ./test.sh)
