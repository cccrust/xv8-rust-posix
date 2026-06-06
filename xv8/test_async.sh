#!/bin/bash
set -e
DIR=$(cd "$(dirname "$0")" && pwd)
cd "$DIR"

echo ""
echo "=== Async / Tokio-Compat Tests ==="
echo "  _async _httpepoll _axum"
echo ""

# Build only the user package (fast, no posix/net cross-compile needed)
cargo build --release --package user

touch /tmp/testmode
echo "async,httpepoll,axum" > /tmp/test_args

# Kernel needs /init as PID 1
required="target/riscv64gc-unknown-none-elf/release/init"

test_bins="$required target/riscv64gc-unknown-none-elf/release/_testrunner"
for name in async httpepoll axum; do
  bin="target/riscv64gc-unknown-none-elf/release/_$name"
  [ -f "$bin" ] && test_bins="$test_bins $bin"
done

if [ -f target/fs.img ]; then mv target/fs.img /tmp/fs.img.backup; fi
rm -f target/fs.img
qemu-img create target/fs.img 256M
HOST=$(rustc -vV | grep host | cut -d' ' -f2)
# Use mkfs binary directly (skip posix/net build)
# shellcheck disable=SC2086
cargo run --release --manifest-path mkfs/Cargo.toml --target "$HOST" -- \
  target/fs.img $test_bins /tmp/test_args /tmp/testmode

if ! cargo run --release; then
    echo "async tests FAILED"
    [ -f /tmp/fs.img.backup ] && mv /tmp/fs.img.backup target/fs.img
    exit 1
fi
echo "async tests PASS"
[ -f /tmp/fs.img.backup ] && mv /tmp/fs.img.backup target/fs.img
