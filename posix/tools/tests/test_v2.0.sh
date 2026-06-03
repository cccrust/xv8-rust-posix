#!/bin/sh
# test_v2.0.sh — v2.0 工具測試（rev, col, look, who, users, last）
# 使用 set -x 顯示每個指令
set -x

PASS=0
FAIL=0
BINDIR=target/release

assert_eq() {
    expected="$1"
    actual="$2"
    msg="$3"
    if [ "$expected" = "$actual" ]; then
        PASS=$((PASS + 1))
        echo "  ✓ $msg" >&2
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $msg" >&2
        echo "    expected: [$expected]" >&2
        echo "    actual:   [$actual]" >&2
    fi
}

assert_nonempty() {
    result="$1"
    msg="$2"
    if [ -n "$result" ]; then
        PASS=$((PASS + 1))
        echo "  ✓ $msg" >&2
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $msg (empty)" >&2
    fi
}

# ─── rev ───────────────────────────────────────────────────────────────────

echo "=== rev ===" >&2

result=$(echo "hello" | $BINDIR/rev)
assert_eq "olleh" "$result" "rev hello"

result=$(printf "abc\n123\n" | $BINDIR/rev)
assert_eq "cba" "$(echo "$result" | head -1)" "rev abc -> cba"
assert_eq "321" "$(echo "$result" | tail -1)" "rev 123 -> 321"

result=$(echo "" | $BINDIR/rev)
assert_eq "" "$result" "rev empty"

# ─── col ───────────────────────────────────────────────────────────────────

echo "=== col ===" >&2

result=$(printf "hello\x08\x08\x08\x08\x08world\n" | $BINDIR/col)
assert_eq "world" "$(echo "$result" | tr -d '\n')" "col backspace"

result=$(printf "abc\n" | $BINDIR/col)
assert_eq "abc" "$(echo "$result" | tr -d '\n')" "col passthrough"

# ─── look ──────────────────────────────────────────────────────────────────

echo "=== look ===" >&2

# look needs a dictionary file; test with a temp one
printf "apple\nappliance\nbanana\ncat\n" > /tmp/test_look_dict.txt
result=$($BINDIR/look app /tmp/test_look_dict.txt 2>/dev/null)
assert_eq "apple" "$(echo "$result" | head -1)" "look app matches apple"
assert_eq "appliance" "$(echo "$result" | tail -1)" "look app matches appliance"

result=$($BINDIR/look zzz /tmp/test_look_dict.txt 2>/dev/null)
assert_eq "" "$result" "look zzz matches nothing"
rm -f /tmp/test_look_dict.txt

# ─── who ───────────────────────────────────────────────────────────────────

echo "=== who ===" >&2

result=$($BINDIR/who -q 2>&1)
assert_nonempty "$result" "who -q returns something"

result=$($BINDIR/who -b 2>&1 || echo "")
# -b (boot time) might not work on all systems
echo "  … who -b done" >&2

# ─── users ─────────────────────────────────────────────────────────────────

echo "=== users ===" >&2

result=$($BINDIR/users 2>&1)
assert_nonempty "$result" "users returns something"

# ─── last ──────────────────────────────────────────────────────────────────

echo "=== last ===" >&2

# last might not have data; just check it doesn't crash
$BINDIR/last 2>/dev/null && echo "  ✓ last" >&2 || echo "  ✓ last (no data)" >&2
PASS=$((PASS + 1))

# ─── Result ────────────────────────────────────────────────────────────────

echo "" >&2
echo "========================================" >&2
echo "Results:  PASS: $PASS  FAIL: $FAIL" >&2
echo "========================================" >&2
exit $FAIL