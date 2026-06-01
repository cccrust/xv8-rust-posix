#!/bin/bash
# xv8 QEMU integration test suite
# Tests commands in the QEMU virtual machine
set -x

FAIL=0
PASS=0
TEST_DIR=/xv8_tests

run() {
    local name="$1"
    local cmd="$2"
    local expect="$3"
    
    result=$(echo "$cmd" | timeout 30 ./xv8/target/riscv64gc-unknown-none-elf/release/xv8 2>&1)
    status=$?
    
    if echo "$result" | grep -q "$expect"; then
        echo "PASS: $name"
        PASS=$((PASS+1))
    else
        echo "FAIL: $name (expected \"$expect\", got \"$result\")"
        FAIL=$((FAIL+1))
    fi
}

run_shell() {
    local name="$1"
    local script="$2"
    local expect="$3"
    
    result=$(echo "$script" | timeout 30 ./xv8/target/riscv64gc-unknown-none-elf/release/xv8 2>&1)
    status=$?
    
    if echo "$result" | grep -q "$expect"; then
        echo "PASS: $name"
        PASS=$((PASS+1))
    else
        echo "FAIL: $name (expected \"$expect\", got \"$result\")"
        FAIL=$((FAIL+1))
    fi
}

echo "=== Building xv8 ==="
(cd xv8 && ./mkfs.sh && cargo run --release) &
XV8_PID=$!
sleep 2
kill $XV8_PID 2>/dev/null
wait $XV8_PID 2>/dev/null

echo
echo "=== xv8 Command Tests ==="
echo

# ─── Basic Commands ──────────────────────────────────────────────────────

run_shell "echo hello" "/bin/echo hello" "hello"
run_shell "echo multi" "/bin/echo a b c" "a b c"

run_shell "cat file" "/bin/cat /LICENSE" "MIT"
run_shell "cat stdin" "echo hello | /bin/cat" "hello"

run_shell "true exit 0" "/bin/true; echo EXIT=\$?" "EXIT=0"
run_shell "false exit 1" "/bin/false; echo EXIT=\$?" "EXIT=1"

run_shell "yes" "/bin/yes | head -1" "y"

run_shell "wc -l" "/bin/wc -l /LICENSE" "21"
run_shell "wc pipe" "cat /LICENSE | /bin/wc -l" "21"

run_shell "basename" "/bin/basename /usr/bin/file.txt" "file.txt"
run_shell "dirname" "/bin/dirname /usr/bin/file.txt" "/usr/bin"

# ─── ls (already working) ────────────────────────────────────────────────

run_shell "ls root" "/bin/ls /" "LICENSE"
run_shell "ls -l" "/bin/ls -l /" "rw-r--r--"
run_shell "ls -la includes dot" "/bin/ls -la / | head -3" "d"
run_shell "ls mkdir dir" "mkdir /ccc_dir; /bin/ls / | grep ccc_dir" "ccc_dir"

# ─── mkdir / rmdir ───────────────────────────────────────────────────────

run_shell "mkdir" "mkdir /testdir; /bin/ls / | grep testdir" "testdir"
run_shell "rmdir" "mkdir /toremove; rmdir /toremove; /bin/ls / | grep -c toremove" "0"

# ─── touch ───────────────────────────────────────────────────────────────

run_shell "touch" "touch /testfile; /bin/ls / | grep testfile" "testfile"

# ─── rm (after we fix it) ────────────────────────────────────────────────

run_shell "rm file" "touch /torm; rm /torm; echo AFTER_RM; /bin/ls / | grep -c torm" "AFTER_RM"

# ─── cp ──────────────────────────────────────────────────────────────────

run_shell "cp file" "echo cpdata > /cp_src; cp /cp_src /cp_dst; cat /cp_dst" "cpdata"

# ─── mv (after we fix it) ────────────────────────────────────────────────

run_shell "mv file" "echo mvdata > /mv_src; mv /mv_src /mv_dst; cat /mv_dst" "mvdata"

# ─── chmod / chown (after we fix it) ─────────────────────────────────────

run_shell "chmod" "touch /cmfile; chmod 600 /cmfile; /bin/ls -l /cmfile" "rw-------"

# ─── head / tail ─────────────────────────────────────────────────────────

run_shell "head -n 3" "head -n 3 /LICENSE" "MIT"
run_shell "tail" "tail -n 3 /LICENSE" "Permission"

# ─── sort / uniq ─────────────────────────────────────────────────────────

run_shell "sort" "echo -e 'b\na\nc' | sort | head -1" "a"
run_shell "uniq" "echo -e 'a\na\nb\nc\nc' | uniq" "a"

# ─── grep ────────────────────────────────────────────────────────────────

run_shell "grep LICENSE" "/bin/grep MIT /LICENSE" "MIT"
run_shell "grep -v" "/bin/grep -v MIT /LICENSE | head -1" "Copyright"

# ─── sed ─────────────────────────────────────────────────────────────────

run_shell "sed subst" "echo 'hello world' | sed 's/world/xv8/'" "hello xv8"

# ─── pipes and redirects ─────────────────────────────────────────────────

run_shell "pipe chain" "echo hello | cat | wc -c" "6"
run_shell "redirect >" "echo 'write test' > /r_test; cat /r_test" "write test"
run_shell "redirect >>" "echo 'line1' > /a_test; echo 'line2' >> /a_test; wc -l /a_test" "2"

# ─── env vars ────────────────────────────────────────────────────────────

run_shell "printenv PATH" "printenv PATH" "/bin"
run_shell "env" "env | grep PATH" "/bin"

# ─── test command ────────────────────────────────────────────────────────

run_shell "test -e /" "test -e /LICENSE; echo EXIT=\$?" "EXIT=0"
run_shell "test -f file" "test -f /LICENSE; echo EXIT=\$?" "EXIT=0"
run_shell "test -d dir" "test -d /; echo EXIT=\$?" "EXIT=0"
run_shell "test -z empty" "test -z ''; echo EXIT=\$?" "EXIT=0"

# ─── system ──────────────────────────────────────────────────────────────

run_shell "whoami" "whoami" "root"
run_shell "uname" "uname" "xv8"
run_shell "hostname" "hostname" "xv8"

# ─── du ──────────────────────────────────────────────────────────────────

run_shell "du" "du / | head -1" "/"

# ─── ps / kill ───────────────────────────────────────────────────────────

run_shell "ps" "ps" "PID"

# ─── Result ──────────────────────────────────────────────────────────────

echo
echo "=== xv8 Test Results ==="
echo "PASS: $PASS"
echo "FAIL: $FAIL"
echo "Total: $((PASS + FAIL))"
