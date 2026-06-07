#!/bin/bash
set -e
DIR=$(cd "$(dirname "$0")" && pwd)
cd "$DIR"

echo ""
echo "=== Core Kernel Tests ==="
echo "  _fs _pipe _proc _fd _sbrk _cow _syscall _thread _thread_v3 _eventfd _memfd_create _pidfd"
echo ""

# Build only the user package (fast, no posix/net cross-compile needed)
cargo build --release --package user

touch /tmp/testmode
echo "fs,pipe,proc,fd,sbrk,cow,syscall,thread,thread_v3,eventfd,memfd_create,pidfd" > /tmp/test_args

# Kernel needs /init as PID 1
required="target/riscv64gc-unknown-none-elf/release/init"

test_bins="$required target/riscv64gc-unknown-none-elf/release/_testrunner"
for name in fs pipe proc fd sbrk cow syscall thread thread_v3 eventfd memfd_create pidfd; do
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
    echo "core tests FAILED"
    [ -f /tmp/fs.img.backup ] && mv /tmp/fs.img.backup target/fs.img
    exit 1
fi
echo "core tests PASS"
[ -f /tmp/fs.img.backup ] && mv /tmp/fs.img.backup target/fs.img
