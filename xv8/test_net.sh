#!/bin/bash
set -e
DIR=$(cd "$(dirname "$0")" && pwd)
cd "$DIR"

echo ""
echo "=== Networking Tests ==="
echo "  _net _neteth _netdns _tcpecho _nettools _http"
echo ""
echo "  Requires POSIX tools: httpd httpget tcpserver tcpclient"
echo ""

touch /tmp/testmode
echo "net,neteth,netdns,tcpecho,nettools,http" > /tmp/test_args

test_bins="target/riscv64gc-unknown-none-elf/release/_testrunner"
for name in net neteth netdns tcpecho nettools http; do
  bin="target/riscv64gc-unknown-none-elf/release/_$name"
  [ -f "$bin" ] && test_bins="$test_bins $bin"
done

if [ -f target/fs.img ]; then mv target/fs.img /tmp/fs.img.backup; fi
rm -f target/fs.img
# shellcheck disable=SC2086
./mkfs.sh $test_bins /tmp/test_args /tmp/testmode

if ! cargo run --release; then
    echo "net tests FAILED"
    [ -f /tmp/fs.img.backup ] && mv /tmp/fs.img.backup target/fs.img
    exit 1
fi
echo "net tests PASS"
[ -f /tmp/fs.img.backup ] && mv /tmp/fs.img.backup target/fs.img
