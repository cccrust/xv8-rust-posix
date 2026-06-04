#!/bin/bash
set -uo pipefail

repo_dir=$(cd "$(dirname "$0")" && pwd)
HOST_TARGET=$(rustc -vV | grep host | cut -d' ' -f2)
PASS=0
FAIL=0

pass() { PASS=$((PASS+1)); echo "PASS: $1"; }
fail() { FAIL=$((FAIL+1)); echo "FAIL: $1"; }

cleanup() {
    if [ -f "$repo_dir/posix/.cargo/config.toml.bak" ]; then
        mv "$repo_dir/posix/.cargo/config.toml.bak" "$repo_dir/posix/.cargo/config.toml"
    fi
}
trap cleanup EXIT

echo "========================================="
echo "  xv8-rust-posix Full Test Suite"
echo "  Host: $HOST_TARGET"
echo "========================================="

# ─── 1. POSIX host build & tests ─────────────────
echo ""
echo "=== Phase 1: POSIX Host Tests ==="

if [ -f "$repo_dir/posix/.cargo/config.toml" ]; then
    mv "$repo_dir/posix/.cargo/config.toml" "$repo_dir/posix/.cargo/config.toml.bak"
fi

if cargo build --release --manifest-path "$repo_dir/posix/Cargo.toml" --target "$HOST_TARGET" 2>&1; then
    pass "posix host build"
else
    fail "posix host build"
fi

HOST_RELEASE="$repo_dir/posix/target/$HOST_TARGET/release"
if [ -d "$HOST_RELEASE" ]; then
    if PATH="$HOST_RELEASE:$PATH" sh "$repo_dir/posix/tools/tests/test_sh_basic.sh" 2>&1; then
        pass "posix shell tests (33)"
    else
        fail "posix shell tests"
    fi

    if PATH="$HOST_RELEASE:$PATH" sh "$repo_dir/posix/tools/tests/test_tools_core.sh" 2>&1; then
        pass "posix core tools tests (21)"
    else
        fail "posix core tools tests"
    fi
fi

# ─── 2. POSIX cargo test ─────────────────────────
echo ""
echo "=== POSIX cargo test ==="
if cargo test --release --manifest-path "$repo_dir/posix/Cargo.toml" --target "$HOST_TARGET" 2>&1; then
    pass "posix cargo test"
else
    fail "posix cargo test"
fi

# restore posix cross-compile config
if [ -f "$repo_dir/posix/.cargo/config.toml.bak" ]; then
    mv "$repo_dir/posix/.cargo/config.toml.bak" "$repo_dir/posix/.cargo/config.toml"
fi

# ─── 3. Net host tests ────────────────────────────
echo ""
echo "=== Phase 2: Net Host Tests ==="

(cd "$repo_dir/net" && ./test.sh 2>&1) && pass "net host tests" || fail "net host tests"

# ─── 4. Cross-compilation ────────────────────────
echo ""
echo "=== Phase 3: Cross-Compilation ==="

if cargo build --release --manifest-path "$repo_dir/posix/Cargo.toml" --target riscv64gc-unknown-none-elf --no-default-features 2>&1; then
    pass "posix riscv64 cross-compile"
else
    fail "posix riscv64 cross-compile"
fi

if cargo build --release --manifest-path "$repo_dir/xv8rust/Cargo.toml" --target riscv64gc-unknown-none-elf 2>&1; then
    pass "xv8rust riscv64 cross-compile"
else
    fail "xv8rust riscv64 cross-compile"
fi

if cargo build --release --manifest-path "$repo_dir/net/Cargo.toml" --package tools \
    --no-default-features --features xv8 -Zbuild-std=core,alloc --target riscv64gc-unknown-none-elf 2>&1; then
    pass "net tools riscv64 cross-compile"
else
    fail "net tools riscv64 cross-compile"
fi

# ─── 5. xv8 QEMU integration tests ────────────────
echo ""
echo "=== Phase 4: xv8 QEMU Integration Tests ==="

(cd "$repo_dir/xv8" && ./test.sh 2>&1) && pass "xv8 QEMU tests" || fail "xv8 QEMU tests"

# ─── Results ─────────────────────────────────────
echo ""
echo "========================================="
echo "  Results: $PASS passed, $FAIL failed"
echo "========================================="
if [ $FAIL -gt 0 ]; then
    exit 1
fi
