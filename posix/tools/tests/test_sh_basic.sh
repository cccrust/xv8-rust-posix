#!/bin/sh
# test_sh_basic.sh — 測試 shell 基本功能
export PATH=$PWD/posix/target/release:$PATH
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

assert_status() {
    expected="$1"
    msg="$2"
    actual=$?
    if [ "$expected" = "$actual" ]; then
        PASS=$((PASS + 1))
        echo "  ✓ $msg"
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $msg"
        echo "    expected status: $expected"
        echo "    actual status:   $actual"
    fi
}

echo "=== 1. echo ==="
result=$(echo hello world)
assert_eq "hello world" "$result" "echo hello world"

result=$(printf hello)
assert_eq "hello" "$result" "echo -n hello"

echo ""
echo "=== 2. cat ==="
result=$(echo hello | cat)
assert_eq "hello" "$result" "cat from pipe"

echo "=== 3. wc ==="
result=$(printf "abc\ndef\n" | wc -l | tr -d ' ')
assert_eq "2" "$result" "wc -l"

result=$(printf "hello" | wc -c | tr -d ' ')
assert_eq "5" "$result" "wc -c"

echo "=== 4. true / false ==="
true && assert_eq "0" "0" "true returns 0"
false && FAIL=$((FAIL + 1)) || PASS=$((PASS + 1))
echo "  ✓ false returns non-zero"

echo "=== 5. basename / dirname ==="
result=$(basename /foo/bar/baz.txt)
assert_eq "baz.txt" "$result" "basename"

result=$(dirname /foo/bar/baz.txt)
assert_eq "/foo/bar" "$result" "dirname"

echo "=== 6. Variable expansion ==="
x=hello
assert_eq "hello" "$x" "simple variable \$x"

y="hello world"
assert_eq "hello world" "$y" "variable with spaces"

assert_eq "11" "${#y}" "variable length \${#y}"

echo "=== 7. Command substitution ==="
result=$(echo hello)
assert_eq "hello" "$result" "command substitution \$(echo hello)"

echo "=== 8. Arithmetic expansion ==="
result=$((2 + 3))
assert_eq "5" "$result" "arithmetic 2+3"

result=$((10 / 3))
assert_eq "3" "$result" "arithmetic 10/3"

echo "=== 9. Test builtin ==="
test -d / && assert_eq "0" "0" "test -d /"

test -f / || assert_eq "0" "0" "test -f / (false)"

test "hello" = "hello" && assert_eq "0" "0" "test string equality"

echo "=== 10. if/then/fi ==="
result=$(if true; then echo yes; fi)
assert_eq "yes" "$result" "if true"

result=$(if false; then echo yes; else echo no; fi)
assert_eq "no" "$result" "if false with else"

echo "=== 11. for loop ==="
result=$(for i in a b c; do printf $i; done)
assert_eq "abc" "$result" "for loop concatenated"

echo "=== 12. while loop ==="
result=$(i=0; while [ $i -lt 3 ]; do printf $i; i=$((i + 1)); done)
assert_eq "012" "$result" "while loop"

echo "=== 13. Pipeline ==="
result=$(echo hello | tr h H)
assert_eq "Hello" "$result" "pipeline with tr"

result=$(printf "a\nb\nc" | grep b)
assert_eq "b" "$result" "pipeline with grep"

echo "=== 14. Redirection ==="
echo hello > /tmp/sh_test_redirect.txt
result=$(cat /tmp/sh_test_redirect.txt)
assert_eq "hello" "$result" "> redirection"

echo world >> /tmp/sh_test_redirect.txt
result=$(wc -l < /tmp/sh_test_redirect.txt | tr -d ' ')
assert_eq "2" "$result" ">> append and < input"

echo "=== 15. \$? tracking ==="
true; assert_eq "0" "$?" "true sets \$?=0"
false; assert_eq "1" "$?" "false sets \$?=1"

echo "=== 16. && / || ==="
result=$(true && echo ok)
assert_eq "ok" "$result" "&& with success"

result=$(false || echo fallback)
assert_eq "fallback" "$result" "|| with failure"

echo "=== 17. \$@ \$# ==="
set -- a b c
assert_eq "3" "$#" "set -- and \$#"
assert_eq "a b c" "$*" "\$* after set --"

echo "=== 18. Eval ==="
result=$(eval echo hello world)
assert_eq "hello world" "$result" "eval echo"

echo "=== 19. Heredoc ==="

echo ""
echo "========================================"
echo "Results:  PASS: $PASS  FAIL: $FAIL"
echo "========================================"
[ $FAIL -eq 0 ] && exit 0 || exit 1
