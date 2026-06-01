#!/bin/bash
set -e

cargo build --release --package user

test_bins=$(find user/testbin/*.rs | sed 's|user/testbin/\(.*\)\.rs|target/riscv64gc-unknown-none-elf/release/_\1|')

touch /tmp/testmode

mv target/fs.img /tmp/fs.img.backup 2>/dev/null || true
qemu-img create target/fs.img 256M

./mkfs.sh $test_bins /tmp/testmode

if ! cargo run --release; then
  echo "test failed"
  mv /tmp/fs.img.backup target/fs.img 2>/dev/null || true
  exit 1
fi

echo "test passed"
mv /tmp/fs.img.backup target/fs.img 2>/dev/null || true
