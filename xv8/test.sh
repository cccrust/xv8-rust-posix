#!/bin/bash
set -e
DIR=$(cd "$(dirname "$0")" && pwd)
cd "$DIR"

PASS=0
FAIL=0
pass() { PASS=$((PASS+1)); echo "===> PASS: $1"; }
fail() { FAIL=$((FAIL+1)); echo "===> FAIL: $1"; }

echo "========================================="
echo "  xv8 QEMU Integration Tests"
echo "========================================="

# Build everything once (cache for subsequent runs)
echo ""
echo "=== Pre-build ==="
echo "Building user + posix + net tools..."
cargo build --release --package user
cargo build --release --manifest-path ../posix/Cargo.toml --package tools --no-default-features
cargo build --release --manifest-path ../net/Cargo.toml --package tools \
  --no-default-features --features xv8 \
  -Zbuild-std=core,alloc --target riscv64gc-unknown-none-elf

# Clean up stale backups
rm -f /tmp/fs.img.backup

# ─── 1. Core Kernel Tests ────────────────────
echo ""
echo "--- test_core.sh ---"
if ./test_core.sh 2>&1 | tail -5; then
    pass "core"
else
    fail "core"
fi

# ─── 2. Networking Tests ──────────────────────
echo ""
echo "--- test_net.sh ---"
rm -f /tmp/fs.img.backup
# Build posix+net if not cached (they were built above; mkfs.sh re-runs but cargo caches)
if ./test_net.sh 2>&1 | tail -5; then
    pass "net"
else
    fail "net"
fi

# ─── 3. Async Tests ────────────────────────────
echo ""
echo "--- test_async.sh ---"
rm -f /tmp/fs.img.backup
if ./test_async.sh 2>&1 | tail -5; then
    pass "async"
else
    fail "async"
fi

# ─── 4. Shell Tests ────────────────────────────
echo ""
echo "--- test_shell.sh ---"
rm -f /tmp/fs.img.backup
if ./test_shell.sh 2>&1 | tail -5; then
    pass "shell"
else
    fail "shell"
fi

# ─── Results ─────────────────────────────────
echo ""
echo "========================================="
echo "  xv8 Tests: $PASS/4 passed, $FAIL failed"
echo "========================================="
rm -f /tmp/testmode /tmp/test_args /tmp/fs.img.backup
if [ $FAIL -gt 0 ]; then exit 1; fi
