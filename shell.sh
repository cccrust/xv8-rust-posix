#!/bin/bash
set -euo pipefail

# Get the repository root directory
repo_dir=$(cd "$(dirname "$0")" && pwd)

# Build and add POSIX tools to PATH
posix_dir="$repo_dir/posix"
cd "$posix_dir"
cargo build --release --package tools
export PATH="$posix_dir/target/release:$PATH"

# Build and add network tools to PATH
net_dir="$repo_dir/net"
cd "$net_dir"
cargo build --release
export PATH="$net_dir/target/release:$PATH"

# Run the POSIX shell
exec "$posix_dir/target/release/sh"