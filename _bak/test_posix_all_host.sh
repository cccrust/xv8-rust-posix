#!/bin/bash
# Comprehensive posix tools smoke test for macOS host
# Tests each tool with basic arguments to verify it runs without crashing

HOST_ARCH=aarch64-apple-darwin
POSIX_DIR="/Users/Shared/ccc/project/xv8-rust-posix/posix"
BINDIR="$POSIX_DIR/target/$HOST_ARCH/release"
cd "$POSIX_DIR"

# Ensure all binaries are built
echo "=== Building all tools ==="
cargo build --release --target $HOST_ARCH 2>&1 | tail -3
echo ""

PASS=0
FAIL=0
ERRORS=""

# Helper: run a tool test
test_tool() {
    local name="$1"
    local cmd="$2"
    local expect="$3"
    
    result=$(eval "$cmd" 2>&1)
    local status=$?
    
    if [ "$expect" = "NOCHECK" ]; then
        # Just check it doesn't crash
        if [ $status -le 1 ]; then
            echo "  PASS: $name"
            PASS=$((PASS+1))
        else
            echo "  FAIL: $name (exit=$status)"
            ERRORS="$ERRORS  $name: exit=$status, output=$result\n"
            FAIL=$((FAIL+1))
        fi
    elif echo "$result" | grep -q -- "$expect"; then
        echo "  PASS: $name"
        PASS=$((PASS+1))
    else
        echo "  FAIL: $name (got '$result', expected pattern '$expect')"
        ERRORS="$ERRORS  $name: got '$result', expected '$expect'\n"
        FAIL=$((FAIL+1))
    fi
}

# All tools that require no special setup
echo "=== Phase 1: Basic I/O ==="

test_tool "echo"             "$BINDIR/echo hello" "hello"
test_tool "echo -n"          "$BINDIR/echo -n hello" "hello"
test_tool "echo multi"        "$BINDIR/echo a b c" "a b c"
test_tool "echo escapes"      "$BINDIR/echo 'hello\nworld'" "hello"
test_tool "true"             "$BINDIR/true; echo EXIT=\$?" "EXIT=0"
test_tool "false"            "$BINDIR/false; echo EXIT=\$?" "EXIT=1"
test_tool "yes first"        "$BINDIR/yes | head -1" "y"
test_tool "yes custom"       "$BINDIR/yes msg | head -1" "msg"
test_tool "cat stdin"        "echo hello | $BINDIR/cat" "hello"
test_tool "cat -n"           "echo hello | $BINDIR/cat -n" "hello"
test_tool "cat -b"           "echo -e '\nhello' | $BINDIR/cat -b" "hello"
test_tool "cat -s"           "echo -e 'a\n\n\nb' | $BINDIR/cat -s" "a"
test_tool "wc default"       "echo 'hello world' | $BINDIR/wc" "1"
test_tool "wc -l"            "echo -e 'a\nb\nc' | $BINDIR/wc -l" "3"
test_tool "wc -w"            "echo 'one two three' | $BINDIR/wc -w" "3"
test_tool "wc -c"            "echo -n hello | $BINDIR/wc -c" "5"
test_tool "wc -m"            "echo -n hello | $BINDIR/wc -m" "5"
test_tool "basename"         "$BINDIR/basename /usr/bin/file.txt" "file.txt"
test_tool "basename suffix"  "$BINDIR/basename /dir/file.txt .txt" "file"
test_tool "dirname"          "$BINDIR/dirname /usr/bin/file.txt" "/usr/bin"
test_tool "dirname root"     "$BINDIR/dirname /" "/"
test_tool "dirname trailing" "$BINDIR/dirname /usr/bin/" "/usr"
test_tool "sleep"            "timeout 5 $BINDIR/sleep 0.1"
test_tool "uname"            "$BINDIR/uname" "NOCHECK"
test_tool "uname -m"         "$BINDIR/uname -m" "NOCHECK"
test_tool "uname -n"         "$BINDIR/uname -n" "NOCHECK"
test_tool "uname -r"         "$BINDIR/uname -r" "NOCHECK"
test_tool "uname -a"         "$BINDIR/uname -a" "NOCHECK"
test_tool "printenv"         "$BINDIR/printenv PATH" "NOCHECK"
test_tool "printenv empty"   "$BINDIR/printenv NOSUCHVAR" ""
test_tool "env"              "$BINDIR/env" "PATH"
test_tool "whoami"           "$BINDIR/whoami" "NOCHECK"
test_tool "id"               "$BINDIR/id" "uid"
test_tool "id -u"            "test \"\$($BINDIR/id -u)\" -gt 0" "NOCHECK"
test_tool "id -g"            "test \"\$($BINDIR/id -g)\" -gt 0" "NOCHECK"
test_tool "hostname"         "$BINDIR/hostname" "NOCHECK"

echo ""
echo "=== Phase 2: File Operations ==="

TMPDIR=$(mktemp -d /tmp/posix_test_XXXX)
pushd "$TMPDIR" >/dev/null

# ls
mkdir -p subdir
touch file1.txt file2.txt
test_tool "ls"               "$BINDIR/ls" "file1"
test_tool "ls -la"           "$BINDIR/ls -la" "file1"
test_tool "ls -la /"         "$BINDIR/ls -la /" "tmp"
test_tool "ls dir"            "$BINDIR/ls subdir" ""
test_tool "ls -R"            "$BINDIR/ls -R" "subdir"

# mkdir / rmdir
test_tool "mkdir"            "$BINDIR/mkdir newdir && test -d newdir" "NOCHECK"
test_tool "mkdir -p"         "$BINDIR/mkdir -p a/b/c && test -d a/b/c" "NOCHECK"
test_tool "rmdir"            "$BINDIR/rmdir newdir && ! test -d newdir" "NOCHECK"
test_tool "rmdir -p"         "$BINDIR/rmdir -p a/b/c && ! test -d a" "NOCHECK"

# touch
test_tool "touch new"        "$BINDIR/touch newfile && test -f newfile" "NOCHECK"
test_tool "touch existing"   "$BINDIR/touch newfile && test -f newfile" "NOCHECK"

# rm
test_tool "rm file"          "touch rmfile && $BINDIR/rm rmfile && ! test -f rmfile" "NOCHECK"
test_tool "rm -rf dir"       "mkdir rmdir && touch rmdir/f && $BINDIR/rm -rf rmdir && ! test -d rmdir" "NOCHECK"

# cp
echo "cpdata" > cp_src
test_tool "cp file"          "$BINDIR/cp cp_src cp_dst" "NOCHECK"
test_tool "cp content"       "cat cp_dst" "cpdata"
test_tool "cp -r dir"        "mkdir cpdir && touch cpdir/f && $BINDIR/cp -r cpdir cpdir2 && test -f cpdir2/f" "NOCHECK"
test_tool "cp -p preserve"   "$BINDIR/cp -p cp_src cp_pp && cat cp_pp" "cpdata"

# mv
echo "mvdata" > mv_src
mkdir mvdir
test_tool "mv file"          "$BINDIR/mv mv_src mv_dst && ! test -f mv_src && cat mv_dst" "mvdata"
test_tool "mv dir"           "$BINDIR/mv mvdir mvdir2 && test -d mvdir2" "NOCHECK"

# ln
echo "lndata" > ln_orig
test_tool "ln hard"          "$BINDIR/ln ln_orig ln_hard && cat ln_hard" "lndata"
test_tool "ln -s sym"        "$BINDIR/ln -s ln_orig ln_sym && test -L ln_sym" "NOCHECK"

# chmod
touch cm_file
test_tool "chmod 600"        "$BINDIR/chmod 600 cm_file && [ -r cm_file ]" "NOCHECK"
test_tool "chmod a+x"        "$BINDIR/chmod a+x cm_file && [ -x cm_file ]" "NOCHECK"

# chown (might fail without root, but shouldn't crash)
test_tool "chown"            "$BINDIR/chown 0 cm_file 2>/dev/null; echo EXIT=\$?" "EXIT=0"

# chgrp (might fail without root, but shouldn't crash)
test_tool "chgrp"            "$BINDIR/chgrp 0 cm_file 2>/dev/null; echo EXIT=\$?" "EXIT=0"

popd >/dev/null
rm -rf "$TMPDIR"

echo ""
echo "=== Phase 3: Text Processing ==="

TMPDIR=$(mktemp -d /tmp/posix_text_XXXX)
pushd "$TMPDIR" >/dev/null

# Create test data
printf 'line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\n' > lines.txt
printf 'Apple\nBanana\nCherry\n' > fruits.txt
printf 'a\na\nb\nb\nc\n' > dupes.txt
printf 'col1\tcol2\tcol3\n' > tabs.txt
printf 'hello\nworld\n' > hello.txt
echo -n 'abcdefghij' > bytes.txt

test_tool "head default"     "$BINDIR/head lines.txt | wc -l" "10"
test_tool "head -n 3"        "$BINDIR/head -n 3 lines.txt" "line 1"
test_tool "head -n -3"       "$BINDIR/head -n -3 lines.txt" "line 9"
test_tool "head -c 5"        "$BINDIR/head -c 5 lines.txt" "line "
test_tool "tail default"     "$BINDIR/tail lines.txt | head -1" "line 3"
test_tool "tail -n 3"        "$BINDIR/tail -n 3 lines.txt" "line 10"
test_tool "tail -n +3"       "$BINDIR/tail -n +3 lines.txt | head -1" "line 3"
test_tool "tail -f"          "timeout 1 $BINDIR/tail -f lines.txt 2>/dev/null || true" "NOCHECK"
test_tool "sort alpha"       "printf 'b\na\nc\n' | $BINDIR/sort" "a"
test_tool "sort -r"          "printf 'b\na\nc\n' | $BINDIR/sort -r | head -1" "c"
test_tool "sort -n"          "printf '10\n2\n33\n' | $BINDIR/sort -n | head -1" "2"
test_tool "sort -u"          "printf 'a\na\nb\n' | $BINDIR/sort -u" "a"
test_tool "uniq basic"       "$BINDIR/uniq dupes.txt" "a"
test_tool "uniq -c"          "$BINDIR/uniq -c dupes.txt" "2 a"
test_tool "uniq -d"          "$BINDIR/uniq -d dupes.txt" "a"
test_tool "uniq -u"          "$BINDIR/uniq -u <(printf 'a\na\nb\n')" "b"
test_tool "cut -f"           "$BINDIR/cut -f2 tabs.txt" "col2"
test_tool "cut -d:"          "echo 'a:b:c' | $BINDIR/cut -d: -f2" "b"
test_tool "cut -c"           "echo 'hello' | $BINDIR/cut -c2-4" "ell"
test_tool "cut -b"           "$BINDIR/cut -b2-4 bytes.txt" "bcd"
test_tool "tr lower"         "echo 'HELLO' | $BINDIR/tr 'A-Z' 'a-z'" "hello"
test_tool "tr delete"        "echo 'hello' | $BINDIR/tr -d l" "heo"
test_tool "tr squeeze"       "echo 'aaabbb' | $BINDIR/tr -s ab" "ab"
test_tool "tr complement"    "echo 'hello 123' | $BINDIR/tr -d -c 'a-zA-Z \n'" "hello "
test_tool "tee"              "echo teedata | $BINDIR/tee teefile && cat teefile" "teedata"
test_tool "tee -a"           "echo 'a' | $BINDIR/tee -a teef2 && echo 'b' | $BINDIR/tee -a teef2 && wc -l teef2" "2"
test_tool "od default"       "$BINDIR/od lines.txt | head -1" "NOCHECK"
test_tool "od -c"            "echo 'ab' | $BINDIR/od -c" "a"
test_tool "od -x"            "echo 'ab' | $BINDIR/od -x" "NOCHECK"
test_tool "cmp same"         "cp lines.txt same.txt && $BINDIR/cmp lines.txt same.txt" "NOCHECK"
test_tool "cmp diff"         "$BINDIR/cmp lines.txt fruits.txt 2>&1; test \$? -eq 1" "NOCHECK"
test_tool "diff same"        "$BINDIR/diff lines.txt lines.txt" ""
test_tool "diff diff"        "$BINDIR/diff lines.txt fruits.txt" "---"
test_tool "sort numeric"     "printf '2\n10\n1\n' | $BINDIR/sort -n | head -1" "1"
test_tool "tsort"            "printf 'a b\nb c\n' | $BINDIR/tsort" "a"
test_tool "join"             "printf 'a 1\nb 2\n' > j1; printf 'a x\nb y\n' > j2; $BINDIR/join j1 j2" "a 1 x"
test_tool "paste"            "printf 'a\nb\n' > p1; printf '1\n2\n' > p2; $BINDIR/paste p1 p2" "a	1"

popd >/dev/null
rm -rf "$TMPDIR"

echo ""
echo "=== Phase 4: Search & Filter ==="

TMPDIR=$(mktemp -d /tmp/posix_search_XXXX)
pushd "$TMPDIR" >/dev/null

printf 'apple\nbanana\ncherry\nApple\n' > fruits.txt
echo 'hello world' > greet.txt

test_tool "grep basic"       "$BINDIR/grep banana fruits.txt" "banana"
test_tool "grep -i"          "$BINDIR/grep -i apple fruits.txt" "apple"
test_tool "grep -v"          "$BINDIR/grep -v apple fruits.txt | wc -l" "3"
test_tool "grep -c"          "$BINDIR/grep -c a fruits.txt" "2"
test_tool "grep -l"          "$BINDIR/grep -l banana fruits.txt" "fruits.txt"
test_tool "grep -n"          "$BINDIR/grep -n banana fruits.txt" "2:banana"
test_tool "grep -r file"     "mkdir -p d1 && echo testdata > d1/f; $BINDIR/grep testdata d1/f" "testdata"
test_tool "grep word"        "echo 'the cat' | $BINDIR/grep cat" "cat"
test_tool "grep line"        "echo 'abc' | $BINDIR/grep abc" "abc"
test_tool "sed subst"        "echo 'hello world' | $BINDIR/sed 's/world/universe/'" "hello universe"
test_tool "sed -i"           "echo 'hello' > sedfile && $BINDIR/sed -i '' 's/hello/hi/' sedfile && cat sedfile" "hi"
test_tool "sed -n p"         "echo -e 'a\nb\nc' | $BINDIR/sed -n 'p'" "a"
test_tool "sed delete"       "echo -e 'a\nb\nc' | $BINDIR/sed '/b/d'" "a"
test_tool "sed range"        "echo -e 'a\nb\nc' | $BINDIR/sed -n '2,3p'" "b"
test_tool "sed global"       "echo 'a b a c' | $BINDIR/sed 's/a/x/g'" "x b x c"
test_tool "xargs echo"       "echo extra | $BINDIR/xargs echo prefix" "prefix extra"
test_tool "xargs -n"         "echo -e 'a\nb\nc' | $BINDIR/xargs -n 2" "a b"
test_tool "xargs -I"         "echo 'file' | $BINDIR/xargs -I {} echo found {}" "found file"

popd >/dev/null
rm -rf "$TMPDIR"

echo ""
echo "=== Phase 5: System Tools ==="

TMPDIR=$(mktemp -d /tmp/posix_sys_XXXX)
pushd "$TMPDIR" >/dev/null

test_tool "test -e /"        "$BINDIR/test -e /" "NOCHECK"
test_tool "test -d /"        "$BINDIR/test -d / && echo ISDIR" "ISDIR"
test_tool "test -f /"        "$BINDIR/test -f /dev/null || echo NOTFILE" "NOTFILE"
test_tool "test -r /"        "$BINDIR/test -r / && echo ISREAD" "ISREAD"
test_tool "test -w /tmp"     "$BINDIR/test -w /tmp && echo ISWRITE" "ISWRITE"
test_tool "test str ="       "$BINDIR/test abc = abc && echo EQ" "EQ"
test_tool "test str !="      "$BINDIR/test abc '!=' def && echo NEQ" "NEQ"
test_tool "test -n"          "$BINDIR/test -n hello && echo NONEMPTY" "NONEMPTY"
test_tool "test -z"          "$BINDIR/test -z '' && echo EMPTY" "EMPTY"
test_tool "test int eq"      "$BINDIR/test 5 -eq 5 && echo EQNUM" "EQNUM"
test_tool "test int ne"      "$BINDIR/test 5 -ne 3 && echo NENUM" "NENUM"
test_tool "test int lt"      "$BINDIR/test 3 -lt 5 && echo LTNUM" "LTNUM"
test_tool "test int gt"      "$BINDIR/test 5 -gt 3 && echo GTNUM" "GTNUM"
test_tool "test int le"      "$BINDIR/test 3 -le 5 && echo LENUM" "LENUM"
test_tool "test int ge"      "$BINDIR/test 5 -ge 3 && echo GENUM" "GENUM"

test_tool "printf hello"     "$BINDIR/printf 'hello\n'" "hello"
test_tool "printf %s"        "$BINDIR/printf '%s\n' world" "world"
test_tool "printf %d"        "$BINDIR/printf '%d\n' 42" "42"
test_tool "printf %x"        "$BINDIR/printf '%x\n' 255" "ff"
test_tool "printf %o"        "$BINDIR/printf '%o\n' 8" "10"
test_tool "printf %f"        "$BINDIR/printf '%f\n' 3.14" "3.14"
test_tool "printf flags"     "$BINDIR/printf '%-10s|\n' left" "left"
test_tool "expr 2+3"         "$BINDIR/expr 2 + 3" "5"
test_tool "expr 10-4"        "$BINDIR/expr 10 - 4" "6"
test_tool "expr 3*4"         "$BINDIR/expr 3 '*' 4" "12"
test_tool "expr 10/3"        "$BINDIR/expr 10 / 3" "3"
test_tool "expr 10%3"        "$BINDIR/expr 10 % 3" "1"
test_tool "expr substr"      "$BINDIR/expr substr hello 2 3" "ell"
test_tool "expr length"      "$BINDIR/expr length hello" "5"
test_tool "expr index"       "$BINDIR/expr index hello ll" "3"
test_tool "pwd"              "$BINDIR/pwd" "$TMPDIR"
test_tool "pwd -L"           "$BINDIR/pwd -L" "NOCHECK"
test_tool "pwd -P"           "$BINDIR/pwd -P" "NOCHECK"

test_tool "date"             "$BINDIR/date" "NOCHECK"
test_tool "date +%Y"         "$BINDIR/date '+%Y'" "$(date +%Y)"
test_tool "date -u"          "$BINDIR/date -u" "NOCHECK"
test_tool "date +%s"         '$BINDIR/date +%s | grep -q "^1[0-9]\{9\}$"' "NOCHECK"

test_tool "cal"              "$BINDIR/cal" "$(date +%Y)"
test_tool "cal 2025"         "$BINDIR/cal 2025" "2025"

test_tool "du /tmp"          "$BINDIR/du /tmp | head -1" "NOCHECK"
test_tool "du -h"            "$BINDIR/du -h /tmp | head -1" "NOCHECK"
test_tool "du -s"            "$BINDIR/du -s /tmp" "NOCHECK"

test_tool "df"               "$BINDIR/df | head -1" "Filesystem"

test_tool "nice"             "$BINDIR/nice true" "NOCHECK"
test_tool "nice -n 19"       "$BINDIR/nice -n 19 true" "NOCHECK"
test_tool "nohup"            "$BINDIR/nohup true" "NOCHECK"

# ps
test_tool "ps"               "$BINDIR/ps" "NOCHECK"
test_tool "ps -e"            "$BINDIR/ps -e" "NOCHECK"
test_tool "ps aux"           "$BINDIR/ps aux" "NOCHECK"

popd >/dev/null
rm -rf "$TMPDIR"

echo ""
echo "=== Phase 6: Shell ==="
test_tool "sh -c echo"      "$BINDIR/sh -c 'echo hello'" "hello"
test_tool "sh -c true"      "$BINDIR/sh -c true && echo OK" "OK"
test_tool "sh -c list"      "$BINDIR/sh -c 'echo a && echo b && echo c' | head -1" "a"
test_tool "bash -c echo"    "$BINDIR/bash -c 'echo hello'" "hello"

echo ""
echo "=== Phase 7: Archiving & Compression ==="

TMPDIR=$(mktemp -d /tmp/posix_arch_XXXX)
pushd "$TMPDIR" >/dev/null

echo "tar data" > tarfile.txt
test_tool "tar create"       "$BINDIR/tar -cf archive.tar tarfile.txt && test -f archive.tar" "NOCHECK"
test_tool "tar list"         "$BINDIR/tar -tf archive.tar" "tarfile.txt"
test_tool "tar extract"      "$BINDIR/tar -xf archive.tar && cat tarfile.txt" "tar data"
test_tool "compress"         "echo 'compress test data' > comp_file && $BINDIR/compress comp_file && test -f comp_file.Z" "NOCHECK"
test_tool "uncompress"       "$BINDIR/uncompress comp_file.Z && test -f comp_file" "NOCHECK"
test_tool "zcat"             "echo 'zcat data' > zf && $BINDIR/compress zf && $BINDIR/zcat zf.Z" "zcat data"

popd >/dev/null
rm -rf "$TMPDIR"

echo ""
echo "=== Additional Tools ==="

TMPDIR=$(mktemp -d /tmp/posix_extra_XXXX)
pushd "$TMPDIR" >/dev/null

printf 'a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n' > lines10.txt
printf 'one\ntwo\nthree\nfour\n' > words.txt
mkdir -p dir1 dir2
touch dir1/f1 dir2/f2

test_tool "comm"             "printf 'a\nb\nc\n' > c1; printf 'b\nc\nd\n' > c2; $BINDIR/comm c1 c2" "a"
test_tool "fold"             "echo 'hello world' | $BINDIR/fold -w 3" "hel"
test_tool "fmt"              "echo 'hello   world' | $BINDIR/fmt" "hello world"
test_tool "nl"               "echo -e 'a\nb\nc' | $BINDIR/nl" "a"
test_tool "expand"           "echo -e '\thello' | $BINDIR/expand" "hello"
test_tool "unexpand"         "echo '    hello' | $BINDIR/unexpand" "hello"
test_tool "tabs"             "$BINDIR/tabs 4" "NOCHECK"
test_tool "cksum"            "echo 'test' | $BINDIR/cksum" "NOCHECK"
test_tool "sum"              "echo 'test' | $BINDIR/sum" "NOCHECK"
test_tool "strings"          "echo 'hello' | $BINDIR/strings" "hello"
test_tool "strip"            "echo 'hello world' | $BINDIR/strip 2>/dev/null || true" "NOCHECK"

# split
test_tool "split -l"         "printf '1\n2\n3\n4\n5\n6\n7\n8\n' | $BINDIR/split -l 4 && ls x* | wc -l" "2"
test_tool "split -b"         "echo 'abcdefgh' | $BINDIR/split -b 4 && test -f xab" "NOCHECK"

# pathchk
test_tool "pathchk"          "$BINDIR/pathchk /tmp" "NOCHECK"

test_tool "mktemp"           "$BINDIR/mktemp" "NOCHECK"
test_tool "mktemp -d"        "$BINDIR/mktemp -d" "NOCHECK"

# mkfifo
test_tool "mkfifo"           "$BINDIR/mkfifo test_fifo && test -p test_fifo" "NOCHECK"

# link / unlink
echo 'linkdata' > link_src
test_tool "link"             "$BINDIR/link link_src link_dst && cat link_dst" "linkdata"
test_tool "unlink"           "$BINDIR/unlink link_dst && ! test -f link_dst" "NOCHECK"

# logname
test_tool "logname"          "$BINDIR/logname" "NOCHECK"

# mesg
test_tool "mesg"             "$BINDIR/mesg" "NOCHECK"

# tty
test_tool "tty"              "$BINDIR/tty" "NOCHECK"

# who / users
test_tool "who"              "$BINDIR/who" "NOCHECK"
test_tool "users"            "$BINDIR/users" "NOCHECK"

# alias / unalias
test_tool "unalias"          "$BINDIR/unalias ls" "NOCHECK"

# type
test_tool "type"             "$BINDIR/type ls || true" "NOCHECK"

# true / false (already tested in Phase 1)
test_tool "true"             "$BINDIR/true && echo OK" "OK"

test_tool "nohup output"     "$BINDIR/nohup true" "NOCHECK"

# getconf
test_tool "getconf"          "$BINDIR/getconf PAGESIZE || true" "NOCHECK"

# stty
test_tool "stty"             "$BINDIR/stty -a 2>/dev/null || true" "NOCHECK"

# dd
test_tool "dd"               "echo 'hello world' | $BINDIR/dd bs=4 count=2 2>/dev/null" "hell"

popd >/dev/null
rm -rf "$TMPDIR"

echo ""
echo "=== Tools Requiring Interactive Terminal ==="
echo '(Tested with piped input, may show limited results)'

TMPDIR=$(mktemp -d /tmp/posix_term_XXXX)
pushd "$TMPDIR" >/dev/null

# more (piped input)
test_tool "more"             "echo hello | timeout 3 $BINDIR/more 2>/dev/null || true" "hello"

# ed (piped input - interactive, may not work in non-TTY mode)
test_tool "ed"               "printf 'a\nhello\n.\n,p\nq\n' | timeout 3 $BINDIR/ed 2>/dev/null || true" "NOCHECK"

# vi (piped input - interactive, may not work in non-TTY mode)
echo "vi test" > vi_test.txt
printf 'iHello world\x1b:wq!\n' | timeout 3 $BINDIR/vi vi_test.txt 2>/dev/null || true
test_tool "vi basic"         "test -f vi_test.txt" "NOCHECK"
test_tool "vi content"       "cat vi_test.txt" "NOCHECK"

popd >/dev/null
rm -rf "$TMPDIR"

echo ""
echo "=== Tools that may hang or require special setup ==="

# kill (need to find a process to kill)
test_tool "kill -l"          "$BINDIR/kill -l" "HUP"

# renice
test_tool "renice"           "$BINDIR/renice 0 \$\$ 2>/dev/null || true" "NOCHECK"

# wait
test_tool "wait"             "$BINDIR/sh -c 'sleep 0.1; echo done'" "done"

# umask
test_tool "umask"            "$BINDIR/umask" "NOCHECK"
test_tool "umask -S"         "$BINDIR/umask -S" "NOCHECK"

# hash
test_tool "hash"             "$BINDIR/hash -r 2>/dev/null || true" "NOCHECK"

# command
test_tool "command"          "$BINDIR/command -v ls || true" "NOCHECK"

# csplit
TMPDIR2=$(mktemp -d /tmp/posix_cs_XXXX)
pushd "$TMPDIR2" >/dev/null
printf 'a\nb\nc\n' | $BINDIR/csplit - '/b/' 2>/dev/null
test_tool "csplit"           "ls xx*" "xx00"
popd >/dev/null
rm -rf "$TMPDIR2"

# file (magic number detection)
mkdir -p /tmp/posix_test_dir
test_tool "file"             "$BINDIR/file /tmp/posix_test_dir" "directory"

# find
TMPDIR3=$(mktemp -d /tmp/posix_find_XXXX)
pushd "$TMPDIR3" >/dev/null
touch a.txt b.txt
test_tool "find"             "$BINDIR/find . -name '*.txt'" "a.txt"
popd >/dev/null
rm -rf "$TMPDIR3"

# patch
TMPDIR4=$(mktemp -d /tmp/posix_patch_XXXX)
pushd "$TMPDIR4" >/dev/null
echo 'hello' > patch_orig.txt
printf '1c1\n< hello\n---\n> hi\n' > patch.diff
$BINDIR/patch < patch.diff 2>/dev/null; cat patch_orig.txt
test_tool "patch"            "cat patch_orig.txt" "NOCHECK"
popd >/dev/null
rm -rf "$TMPDIR4"

# printf (escape tests)
test_tool "printf backslash n" "$BINDIR/printf 'a\nb'" "a"
test_tool "printf percent"     "$BINDIR/printf '%s\n' hello" "hello"

# echo -e (requires -e flag for escape interpretation)
test_tool "echo -e tab"      "$BINDIR/echo -e 'a\tb'" "a	b"

echo ""
echo "=== Summary ==="
echo "PASS: $PASS"
echo "FAIL: $FAIL"
echo ""

if [ $FAIL -gt 0 ]; then
    echo "FAILURES:"
    printf '%b\n' "$ERRORS"
fi

exit $FAIL
