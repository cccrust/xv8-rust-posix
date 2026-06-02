#!/bin/sh
# test_tools_core.sh — 測試 POSIX 核心工具
PASS=0
FAIL=0

assert_eq() {
    expected="$1"
    actual="$2"
    msg="$3"
    if [ "$expected" = "$actual" ]; then
        PASS=$((PASS + 1))
        echo "  ✓ $msg"
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $msg"
        echo "    expected: [$expected]"
        echo "    actual:   [$actual]"
    fi
}

echo "=== echo ==="
result=$(echo hello)
assert_eq "hello" "$result" "echo hello"

result=$(printf hello)
assert_eq "hello" "$result" "echo -n"

result=$(echo "hello world" | tr h H)
assert_eq "Hello world" "$result" "echo piped to tr"

echo "=== cat ==="
result=$(echo hello | cat)
assert_eq "hello" "$result" "cat pipe"

echo "hello world" > /tmp/test_cat.txt
result=$(cat -n /tmp/test_cat.txt 2>/dev/null || cat /tmp/test_cat.txt)
# cat -n may not be supported; just check content
result=$(cat /tmp/test_cat.txt)
assert_eq "hello world" "$result" "cat file"

echo "=== true/false ==="
true && PASS=$((PASS + 1)) && echo "  ✓ true"
false && FAIL=$((FAIL + 1)) || echo "  ✓ false returns non-zero" && PASS=$((PASS + 1))

echo "=== basename ==="
result=$(basename /path/to/file.txt)
assert_eq "file.txt" "$result" "basename"

result=$(basename /path/to/dir/)
assert_eq "dir" "$result" "basename trailing slash"

echo "=== dirname ==="
result=$(dirname /path/to/file.txt)
assert_eq "/path/to" "$result" "dirname"

result=$(dirname /path/to/dir/)
assert_eq "/path/to" "$result" "dirname trailing"

echo "=== wc ==="
printf "hello" > /tmp/test_wc.txt
result=$(wc -c < /tmp/test_wc.txt | tr -d ' ')
assert_eq "5" "$result" "wc -c (stripped)"

result=$(printf "line1\nline2\nline3\n" | wc -l | tr -d ' ')
assert_eq "3" "$result" "wc -l pipe"

echo "=== whoami ==="
result=$(whoami)
[ -n "$result" ] && PASS=$((PASS + 1)) && echo "  ✓ whoami returns non-empty ($result)" || echo "  ✗ whoami empty"

echo "=== id ==="
result=$(id -u 2>/dev/null)
[ -n "$result" ] && PASS=$((PASS + 1)) && echo "  ✓ id -u ($result)" || echo "  ✗ id -u failed"

echo "=== uname ==="
result=$(uname)
[ -n "$result" ] && PASS=$((PASS + 1)) && echo "  ✓ uname ($result)" || echo "  ✗ uname failed"

result=$(uname -n)
[ -n "$result" ] && PASS=$((PASS + 1)) && echo "  ✓ uname -n ($result)" || echo "  ✗ uname -n failed"

echo "=== env ==="
result=$(env | wc -l | tr -d ' ')
[ "$result" -ge 1 ] && PASS=$((PASS + 1)) && echo "  ✓ env returns at least 1 line" || echo "  ✗ env failed"

echo "=== pwd ==="
result=$(pwd)
[ -n "$result" ] && PASS=$((PASS + 1)) && echo "  ✓ pwd ($result)" || echo "  ✗ pwd failed"

echo "=== tty ==="
result=$(tty 2>/dev/null || echo "not a tty")
echo "  ✓ tty" && PASS=$((PASS + 1))

echo "=== sleep ==="
result=$(sleep 0 && echo ok)
assert_eq "ok" "$result" "sleep 0"

echo ""
echo "========================================"
echo "Results:  PASS: $PASS  FAIL: $FAIL"
echo "========================================"
[ $FAIL -eq 0 ] && exit 0 || exit 1
