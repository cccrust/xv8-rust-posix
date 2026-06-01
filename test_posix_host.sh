#!/bin/bash
set -x

HOST_TARGET=aarch64-apple-darwin

cd posix
mv .cargo/config.toml .cargo/config.toml.bak 2>/dev/null

PASS=0
FAIL=0

run() {
    local name="$1"; shift
    if "$@"; then
        echo "PASS: $name"
        PASS=$((PASS+1))
    else
        echo "FAIL: $name ($*)" 
        FAIL=$((FAIL+1))
    fi
}

cargo build --release --bin echo   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin true   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin false  --target $HOST_TARGET 2>/dev/null
cargo build --release --bin cat    --target $HOST_TARGET 2>/dev/null
cargo build --release --bin wc     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin basename --target $HOST_TARGET 2>/dev/null
cargo build --release --bin dirname --target $HOST_TARGET 2>/dev/null
cargo build --release --bin sleep  --target $HOST_TARGET 2>/dev/null
cargo build --release --bin uname  --target $HOST_TARGET 2>/dev/null
cargo build --release --bin printenv --target $HOST_TARGET 2>/dev/null
cargo build --release --bin env    --target $HOST_TARGET 2>/dev/null
cargo build --release --bin whoami --target $HOST_TARGET 2>/dev/null
cargo build --release --bin id     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin hostname --target $HOST_TARGET 2>/dev/null
cargo build --release --bin yes    --target $HOST_TARGET 2>/dev/null

BINDIR=target/$HOST_TARGET/release

# ─── Phase 1: Basic I/O ──────────────────────────────────────────────────

run "true exit 0"   test "$($BINDIR/true; echo $?)" = "0"
run "false exit 1"  test "$($BINDIR/false; echo $?)" = "1"

run "echo hello"    test "$($BINDIR/echo hello)" = "hello"
run "echo -n"       test "$($BINDIR/echo -n hello)" = "hello"
run "echo multi"    test "$($BINDIR/echo a b c)" = "a b c"

run "yes first line" test "$($BINDIR/yes | head -1)" = "y"
run "yes custom"    test "$($BINDIR/yes hello | head -1)" = "hello"

run "cat stdin"     test "$(echo hello | $BINDIR/cat)" = "hello"
run "cat file"      bash -c 'echo line1 > /tmp/posix_cat_test && test "$('$BINDIR'/cat /tmp/posix_cat_test)" = "line1"'

run "wc stdin"      test "$(echo -e 'a\nb\nc' | $BINDIR/wc -l)" = "3"
run "wc words"      test "$(echo 'one two three' | $BINDIR/wc -w)" = "3"
run "wc chars"      test "$(echo -n hello | $BINDIR/wc -m)" = "5"
run "wc bytes"      test "$(echo -n hello | $BINDIR/wc -c)" = "5"

run "basename"      test "$($BINDIR/basename /usr/bin/file.txt)" = "file.txt"
run "basename suffix" test "$($BINDIR/basename /dir/file.txt .txt)" = "file"
run "dirname"       test "$($BINDIR/dirname /usr/bin/file.txt)" = "/usr/bin"
run "dirname root"  test "$($BINDIR/dirname /)" = "/"

run "sleep 0"       $BINDIR/sleep 0

run "uname nonempty" test -n "$($BINDIR/uname)"
run "uname -m"      test -n "$($BINDIR/uname -m)"
run "uname -n"      test -n "$($BINDIR/uname -n)"

run "printenv PATH" test -n "$($BINDIR/printenv PATH)"
run "printenv empty" test -z "$($BINDIR/printenv NOSUCHVAR_XYZ)"

run "env has PATH"  $BINDIR/env | grep -q PATH
run "whoami nonempty" test -n "$($BINDIR/whoami)"
run "id -u"         test "$($BINDIR/id -u)" -gt 0
run "id -g"         test "$($BINDIR/id -g)" -gt 0
run "hostname nonempty" test -n "$($BINDIR/hostname)"

# ─── Phase 2: File Operations (host native) ──────────────────────────────

cargo build --release --bin ls     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin mkdir  --target $HOST_TARGET 2>/dev/null
cargo build --release --bin touch  --target $HOST_TARGET 2>/dev/null
cargo build --release --bin rm     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin rmdir  --target $HOST_TARGET 2>/dev/null
cargo build --release --bin cp     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin mv     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin ln     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin chmod  --target $HOST_TARGET 2>/dev/null
cargo build --release --bin chown  --target $HOST_TARGET 2>/dev/null

mkdir -p /tmp/posix_test_dir
pushd /tmp/posix_test_dir >/dev/null
rm -rf *

run "ls -la root"   test "$($BINDIR/ls -la / | head -5 | wc -l)" -gt 0
run "mkdir basic"   $BINDIR/mkdir testdir && test -d testdir
run "touch create"  $BINDIR/touch testfile && test -f testfile
run "cp file"       bash -c 'echo data > src && '$BINDIR'/cp src dst && test "$(cat dst)" = "data"'
run "mv file"       bash -c 'echo moved > src2 && '$BINDIR'/mv src2 dst2 && test -f dst2 && ! test -f src2'
run "rm file"       bash -c 'touch to_rm && '$BINDIR'/rm to_rm && ! test -f to_rm'
run "rmdir dir"     bash -c 'mkdir to_rmdir && '$BINDIR'/rmdir to_rmdir && ! test -d to_rmdir'
run "ln hard"       bash -c 'echo linkdata > orig && '$BINDIR'/ln orig link && test "$(cat link)" = "linkdata"'
run "ln sym"        bash -c 'echo symdata > sorig && '$BINDIR'/ln -s sorig slink && test -L slink'
run "chmod"         bash -c 'touch modfile && '$BINDIR'/chmod 600 modfile && test "$(stat -f %Lp modfile)" = "600"'

popd >/dev/null
rm -rf /tmp/posix_test_dir

# ─── Phase 3: Text Processing (host native) ──────────────────────────────

cargo build --release --bin head   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin tail   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin sort   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin uniq   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin cut    --target $HOST_TARGET 2>/dev/null
cargo build --release --bin tr     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin tee    --target $HOST_TARGET 2>/dev/null
cargo build --release --bin od     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin cmp    --target $HOST_TARGET 2>/dev/null
cargo build --release --bin diff   --target $HOST_TARGET 2>/dev/null

mkdir -p /tmp/posix_text_test
pushd /tmp/posix_text_test >/dev/null
rm -rf *
printf 'line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\n' > lines.txt

run "head default"  test "$($BINDIR/head lines.txt | wc -l)" = "10"
run "head -n 3"     test "$($BINDIR/head -n 3 lines.txt | wc -l)" = "3"
run "tail default"  test "$($BINDIR/tail lines.txt | head -1)" = "line 3"
run "sort alpha"    test "$(printf 'b\na\nc\n' | $BINDIR/sort | head -1)" = "a"
run "sort -r"       test "$(printf 'b\na\nc\n' | $BINDIR/sort -r | head -1)" = "c"
run "uniq basic"    test "$(printf 'a\na\nb\nb\nc\n' | $BINDIR/uniq)" = $'a\nb\nc'
run "cut -f"        test "$(printf 'a\tb\tc\n' | $BINDIR/cut -f2)" = "b"
run "tr basic"      test "$(echo hello | $BINDIR/tr l x)" = "hexxo"
run "tee basic"     bash -c 'echo teedata | '$BINDIR'/tee teefile && test "$(cat teefile)" = "teedata"'
run "od basic"      test -n "$($BINDIR/od lines.txt | head -1)"
run "cmp same"      bash -c 'cp lines.txt same.txt && '$BINDIR'/cmp lines.txt same.txt'
run "cmp diff"      bash -c 'echo diff > other.txt && ! '$BINDIR'/cmp lines.txt other.txt 2>/dev/null'
run "diff same"     bash -c '$BINDIR'/diff lines.txt lines.txt
run "diff diff"     bash -c 'echo xxx > diff2.txt && ! '$BINDIR'/diff lines.txt diff2.txt 2>/dev/null'

popd >/dev/null
rm -rf /tmp/posix_text_test

# ─── Phase 4: Search & Filter (host native) ──────────────────────────────

cargo build --release --bin grep   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin sed    --target $HOST_TARGET 2>/dev/null
cargo build --release --bin xargs  --target $HOST_TARGET 2>/dev/null

mkdir -p /tmp/posix_search_test
pushd /tmp/posix_search_test >/dev/null
rm -rf *
printf 'apple\nbanana\ncherry\n' > fruits.txt

run "grep basic"    test "$($BINDIR/grep anana fruits.txt)" = "banana"
run "grep -v"       test "$($BINDIR/grep -v apple fruits.txt | wc -l)" = "2"
run "grep -c"       test "$($BINDIR/grep -c a fruits.txt)" = "2"
run "grep -i"       test "$(printf 'Apple\nbanana\n' | $BINDIR/grep -i apple)" = "Apple"
run "sed subst"     test "$(echo 'hello world' | $BINDIR/sed 's/world/universe/')" = "hello universe"
run "sed global"    test "$(echo 'a b a c' | $BINDIR/sed 's/a/x/g')" = "x b x c"
run "xargs echo"    test "$(echo 'extra' | $BINDIR/xargs echo prefix)" = "prefix extra"

popd >/dev/null
rm -rf /tmp/posix_search_test

# ─── Phase 5: System Tools (host native) ─────────────────────────────────

cargo build --release --bin test   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin date   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin printf --target $HOST_TARGET 2>/dev/null
cargo build --release --bin expr   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin pwd    --target $HOST_TARGET 2>/dev/null
cargo build --release --bin du     --target $HOST_TARGET 2>/dev/null
cargo build --release --bin nice   --target $HOST_TARGET 2>/dev/null
cargo build --release --bin nohup  --target $HOST_TARGET 2>/dev/null

run "test -e /"     $BINDIR/test -e /
run "test -d /"     $BINDIR/test -d /
run "test -e missing" ! $BINDIR/test -e /nonexistent_xyz
run "test str =="   $BINDIR/test abc = abc
run "test str !="   $BINDIR/test abc !"'"'= def
run "test -n"       $BINDIR/test -n hello
run "test -z"       $BINDIR/test -z ""
run "test int eq"   $BINDIR/test 5 -eq 5
run "test int lt"   $BINDIR/test 3 -lt 5

run "printf hello"  test "$($BINDIR/printf 'hello\n')" = "hello"
run "printf %s"     test "$($BINDIR/printf '%s\n' world)" = "world"
run "printf %d"     test "$($BINDIR/printf '%d\n' 42)" = "42"
run "printf %x"     test "$($BINDIR/printf '%x\n' 255)" = "ff"

run "expr 2+3"      test "$($BINDIR/expr 2 + 3)" = "5"
run "expr 10-4"     test "$($BINDIR/expr 10 - 4)" = "6"
run "expr 3*4"      test "$($BINDIR/expr 3 '*' 4)" = "12"
run "expr 10/3"     test "$($BINDIR/expr 10 / 3)" = "3"

run "pwd /"         test "$($BINDIR/pwd)" != ""

run "date -u"       test -n "$($BINDIR/date -u)"
run "date +%Y"      test "$($BINDIR/date '+%Y')" = "$(date +%Y)"

run "du root"       test -n "$($BINDIR/du / | head -1)"
run "nice"          $BINDIR/nice true

# ─── Result ──────────────────────────────────────────────────────────────

mv .cargo/config.toml.bak .cargo/config.toml 2>/dev/null

echo
echo "=== Test Results ==="
echo "PASS: $PASS"
echo "FAIL: $FAIL"
echo "Total: $((PASS + FAIL))"
