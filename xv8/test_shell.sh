#!/bin/bash
set -e
DIR=$(cd "$(dirname "$0")" && pwd)
cd "$DIR"

echo ""
echo "=== Shell Tests ==="
echo "  _shtest (93 shell behavior tests)"
echo ""
echo "  Requires POSIX tools: /sh"
echo ""

touch /tmp/testmode
echo "shtest" > /tmp/test_args

if [ -f target/fs.img ]; then mv target/fs.img /tmp/fs.img.backup; fi
rm -f target/fs.img
# shellcheck disable=SC2086
./mkfs.sh \
  target/riscv64gc-unknown-none-elf/release/_testrunner \
  target/riscv64gc-unknown-none-elf/release/_shtest \
  /tmp/test_args /tmp/testmode

if ! cargo run --release; then
    echo "shell tests FAILED"
    [ -f /tmp/fs.img.backup ] && mv /tmp/fs.img.backup target/fs.img
    exit 1
fi
echo "shell tests PASS"
[ -f /tmp/fs.img.backup ] && mv /tmp/fs.img.backup target/fs.img
