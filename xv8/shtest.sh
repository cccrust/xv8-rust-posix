#!/bin/sh
# shtest.sh -- POSIX shell behavior test
# Runs on both macOS host and xv8 QEMU

pass=0; fail=0; total=0

check() {
    _desc="$1"; _exp="$2"; _got="$3"
    total=$((total + 1))
    if [ -z "$_exp" ] && [ -z "$_got" ]; then
        pass=$((pass + 1))
        echo "ok $total - $_desc"
    else
        if [ -n "$_exp" ] && [ -n "$_got" ] && [ "$_exp" = "$_got" ]; then
            pass=$((pass + 1))
            echo "ok $total - $_desc"
        else
            fail=$((fail + 1))
            echo "not ok $total - $_desc"
            echo "#   expected: [$_exp]"
            echo "#   actual:   [$_got]"
        fi
    fi
}

check_status() {
    _rc=$?; _desc="$1"; _exp="$2"
    total=$((total + 1))
    if [ "$_exp" = "$_rc" ]; then
        pass=$((pass + 1))
        echo "ok $total - $_desc"
    else
        fail=$((fail + 1))
        echo "not ok $total - $_desc"
        echo "#   expected status: $_exp"
        echo "#   actual status:   $_rc"
    fi
}

check_empty() {
    _desc="$1"; _val="$2"
    total=$((total + 1))
    if [ -z "$_val" ]; then
        pass=$((pass + 1))
        echo "ok $total - $_desc"
    else
        fail=$((fail + 1))
        echo "not ok $total - $_desc"
        echo "#   expected: [empty]"
        echo "#   actual:   [$_val]"
    fi
}

export PATH="/:/bin:/usr/bin:/usr/local/bin"
T=/tmp
mkdir -p $T 2>/dev/null || true
TDIR="$T/shtest.$$"
mkdir -p "$TDIR"

echo "1..auto"
echo "# Shell test suite"
echo ""

echo "### 1. echo"
_r=$(echo hello world);        check "echo hello world" "hello world" "$_r"
_r=$(echo -n hello);           check "echo -n (no newline)" "hello" "$_r"
check_empty "echo (empty)" "$(echo)"
_r=$(echo "hello   world");    check "echo with quoted spaces" "hello   world" "$_r"

echo "### 2. Variable expansion"
x=hello;                         check "simple variable" "hello" "$x"
y="hello    world";              check "variable with spaces" "hello    world" "$y"
z="abcde";                       check "variable length" "5" "${#z}"
z="";                            check_empty "empty variable" "${z}"
unset z;                         check_empty "unset variable" "${z}"

echo "### 3. Variable defaults"
unset x;  check ":- on unset"     "default"  "${x:-default}"
x="";     check ":- on empty"     "default"  "${x:-default}"
x=setval; check ":- on set"       "setval"   "${x:-default}"
unset x;  check ":= on unset"     "assigned" "${x:=assigned}"
          check "variable after :=" "assigned" "$x"
unset x;  check_empty ":+ on unset" "${x:+alternate}"
x=set;    check ":+ on set"       "alternate" "${x:+alternate}"

echo "### 4. Command substitution"
_r=$(echo hello);  check "simple" "hello" "$_r"
_r=$(echo a b c);  check "multiple words" "a b c" "$_r"

echo "### 5. Arithmetic"
check "2+3"       "5"  "$((2+3))"
check "10/3"      "3"  "$((10/3))"
check "2+3*4"     "14" "$((2+3*4))"
check "(2+3)*4"   "20" "$(((2+3)*4))"
a=5; b=3;         check "variables" "8" "$((a+b))"
check "nested"    "6"  "$((1+$((2+3))))"

echo "### 6. Redirections"
OF="$TDIR/out.txt"
echo hello > "$OF"
_r=$(cat "$OF"); check "> redirection" "hello" "$_r"
echo world >> "$OF"
_r=$(tail -n1 "$OF"); check ">> append" "world" "$_r"
_r=$(wc -l < "$OF")
_r2=$(echo "$_r" | tr -d ' ')
check "< stdin" "2" "$_r2"
echo hello > "$OF"
echo again > "$OF"
_r=$(cat "$OF"); check "> truncates" "again" "$_r"

echo "### 7. Pipes"
_r=$(echo hello | cat); check "simple pipe" "hello" "$_r"
_r=$(echo hello world | wc -w)
_r2=$(echo "$_r" | tr -d ' ')
check "pipe wc -w" "2" "$_r2"

echo "### 8. Exit codes"
true;  check_status "true returns 0" 0
false; check_status "false returns 1" 1
true && true;  check_status "true && true" 0
false || true; check_status "false || true" 0

echo "### 9. test / [ builtin"
test -d /;             check_status "test -d /" 0
test "hello" = "hello"; check_status "string equality" 0
test "abc" != "def";   check_status "string inequality" 0
test 5 -eq 5;          check_status "-eq" 0
test 5 -ne 6;          check_status "-ne" 0
test 5 -lt 10;         check_status "-lt" 0
test 10 -gt 5;         check_status "-gt" 0
test 5 -le 5;          check_status "-le" 0
test 5 -ge 5;          check_status "-ge" 0
test -z "";            check_status "-z empty" 0
test -n "x";           check_status "-n nonempty" 0
[ "a" = "a" ];         check_status "[ equality" 0

echo "### 10. if/then/elif/else/fi"
_r=$(if true; then echo yes; fi);            check "if true" "yes" "$_r"
_r=$(if false; then echo yes; else echo no; fi); check "if false+else" "no" "$_r"
_r=$(if false; then echo a; elif true; then echo b; else echo c; fi); check "elif true" "b" "$_r"
_r=$(if false; then echo a; elif false; then echo b; else echo c; fi); check "elif false+else" "c" "$_r"
_r=$(if false; then echo a; elif false; then echo b; elif true; then echo c; else echo d; fi); check "multi elif" "c" "$_r"
x=""; if true; then x="executed"; fi
check "if without subshell" "executed" "$x"

echo "### 11. for loop"
x=""; for i in a b c; do x="${x}${i}"; done
check "for loop over words" "abc" "$x"
x=""; for i in 1 2 3; do x="${x}${i}"; done
check "for loop numeric" "123" "$x"

echo "### 12. while loop"
x=""; i=0
while [ $i -lt 3 ]; do x="${x}${i}"; i=$((i+1)); done
check "while with counter" "012" "$x"
x="not executed"
while false; do x="executed"; done
check "while false (no exec)" "not executed" "$x"

echo "### 13. case"
x=""; case "hello" in hello) x=matched ;; *) x=no ;; esac
check "case exact match" "matched" "$x"
x=""; case "abcdef" in abc*) x=wild ;; *) x=no ;; esac
check "case wildcard" "wild" "$x"
x=""; case "foo" in a|b|c) x=matched ;; *) x=other ;; esac
check "case multi (no match)" "other" "$x"
x=""; case "b" in a|b|c) x=matched ;; *) x=other ;; esac
check "case multi (match)" "matched" "$x"

echo "### 14. && / ||"
_r=$(true && echo ok);            check "&& success" "ok" "$_r"
check_empty "&& fail" "$(false && echo no)"
_r=$(false || echo fallback);     check "|| fail" "fallback" "$_r"
check_empty "|| success" "$(true || echo no)"

echo "### 15. set / unset / shift"
set -- a b c
check '$#' "3" "$#"
check '$*' "a b c" "$*"
check '$1' "a" "$1"
check '$2' "b" "$2"
shift
check 'shift $#' "2" "$#"
check '$1 after shift' "b" "$1"
shift 2
check 'shift 2 $#' "0" "$#"
set -- 1 2 3 4 5
shift 3
check 'shift 3 $#' "2" "$#"
check '$1 after shift 3' "4" "$1"

echo "### 16. type"
_r=$(type echo); check "type echo" "echo is a shell builtin" "$_r"
_r=$(type type); check "type type" "type is a shell builtin" "$_r"

echo "### 17. export"
MYVAR=testval; export MYVAR
check "export basic" "testval" "$MYVAR"
export MYVAR2=inline
check "export with assignment" "inline" "$MYVAR2"

echo "### 18. cd / pwd"
cur=$(pwd)
cd /
_r=$(pwd); check "cd /" "/" "$_r"
cd "$cur"
_r=$(pwd); check "cd back" "$cur" "$_r"

echo "### 19. Functions (non-subshell)"
sum=0; add() { sum=$(( $1 + $2 )); }
add 3 4
check "function with args" "7" "$sum"

echo "### 20. eval"
_r=$(eval echo hello world); check "eval" "hello world" "$_r"

echo "### 21. continue / break"
x=""
for i in 1 2 3 4 5; do
    if [ $i -eq 3 ]; then continue; fi
    if [ $i -eq 5 ]; then break; fi
    x="${x}${i}"
done
check "continue/break" "124" "$x"

echo "### 22. source (.)"
echo "SRCTEST=works" > "$TDIR/source.txt"
. "$TDIR/source.txt"
check "source . file" "works" "$SRCTEST"

echo "### 23. readonly"
readonly ROVAR=immutable
check "readonly variable" "immutable" "$ROVAR"

echo "### 24. Single quotes prevent expansion"
x=hello; _r=$(echo '$x'); check "single quotes prevent expansion" '$x' "$_r"
_r=$(echo 'hello world'); check "single quotes multi-word" "hello world" "$_r"

echo "### 25. test string comparison with empty string"
test "x" = ""; check_status 'test "x" = "" (should fail)' 1
test "" = "x"; check_status 'test "" = "x" (should fail)' 1
test "abc" = "def"; check_status 'test "abc" = "def" (should fail)' 1
test "abc" = "abc"; check_status 'test "abc" = "abc" (should succeed)' 0

echo "### 26. Empty-string arguments"
count_args() { check "empty-string func args (count)" "3" "$#"; }
count_args "" "x" ""
check_empty "empty arg from echo" "$(echo "")"
_r=$(echo ""); check_empty "empty arg from variable" "$_r"

rm -rf "$TDIR"

echo ""
echo "# ========================================"
echo "# $total tests, $pass passed, $fail failed"
echo "# ========================================"
if [ $fail -eq 0 ]; then exit 0; else exit 1; fi
