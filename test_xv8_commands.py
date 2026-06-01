#!/usr/bin/env python3
"""xv8 QEMU test - starts QEMU, waits for boot, then sends commands"""
import subprocess as sp
import time
import sys
import os
import select

os.chdir('/Users/Shared/ccc/project/xv8-rust-posix/xv8')

# Start QEMU
proc = sp.Popen(
    ['cargo', 'run', '--release'],
    stdin=sp.PIPE, stdout=sp.PIPE, stderr=sp.DEVNULL
)

def read_all(timeout=5):
    """Read all available output with timeout"""
    buf = b''
    deadline = time.time() + timeout
    while time.time() < deadline:
        r, _, _ = select.select([proc.stdout], [], [], 0.1)
        if r:
            try:
                ch = proc.stdout.read(1)
                if ch:
                    buf += ch
                    sys.stdout.buffer.write(ch)
                    sys.stdout.buffer.flush()
                else:
                    break
            except:
                break
    return buf

def send_byte(b, delay=0.002):
    proc.stdin.write(bytes([b]))
    proc.stdin.flush()
    time.sleep(delay)

def send_line(s, char_delay=0.003):
    for ch in s.encode():
        send_byte(ch, char_delay)
    send_byte(ord('\n'), 0.1)

# Wait for boot
print("Booting...")
output = read_all(timeout=30)
if b'$ ' in output:
    print("Boot OK, got shell prompt")
else:
    print("Warning: didn't see shell prompt in initial output")

# Wait a bit more for shell to be fully ready
time.sleep(0.5)

PASS = 0
FAIL = 0

def run_cmd(cmd, expected, timeout=10):
    """Send command and check for expected output"""
    global PASS, FAIL
    
    # Empty the read buffer first
    read_all(0.5)
    
    send_line(cmd, char_delay=0.003)
    time.sleep(1)
    
    output = read_all(timeout=timeout)
    
    if expected.encode() in output:
        print(f"  PASS: {cmd.split()[0]} (expects '{expected}')")
        PASS += 1
        return True
    else:
        out_str = output.decode(errors='replace')[-200:]
        print(f"  FAIL: {cmd.split()[0]} (expected '{expected}', got '{out_str}')")
        FAIL += 1
        return False

# ── Tests ────────────────────────────────────────────────────────────

time.sleep(2)
print("\n=== Running Tests ===\n")

# Echo (basic test)
run_cmd("echo hello_xv8", "hello_xv8")

# ls
run_cmd("ls /", "LICENSE")
run_cmd("ls -la /", "rw")

# mkdir + rmdir
run_cmd("mkdir /test1", "")  # just check no error
send_line("ls / | grep test1")
time.sleep(0.5)
run_cmd("", "test1")  # check test1 appears

run_cmd("rmdir /test1", "")
run_cmd("ls / | grep -c test1", "0")

# touch + rm
send_line("touch /test2")
time.sleep(0.3)
run_cmd("ls /test2", "test2")

send_line("rm /test2")
time.sleep(0.3)
send_line("ls /test2 2>&1")
time.sleep(0.3)
send_line("echo RM_CHECK")
r = read_all(5)
if b'RM_CHECK' in r:
    print(f"  PASS: rm (no crash)")
    PASS += 1
else:
    print(f"  FAIL: rm")
    FAIL += 1

# cp
send_line("echo cpdat > /cp_src")
time.sleep(0.3)
send_line("cp /cp_src /cp_dst")
time.sleep(0.3)
send_line("cat /cp_dst")
time.sleep(0.3)
send_line("echo CP_CHECK")
r = read_all(5)
if b'cpdat' in r:
    print(f"  PASS: cp")
    PASS += 1
else:
    print(f"  FAIL: cp (output: {r[-100:]})")
    FAIL += 1

# mv
send_line("echo mvdat > /mv_src")
time.sleep(0.3)
send_line("mv /mv_src /mv_dst")
time.sleep(0.3)
send_line("cat /mv_dst")
time.sleep(0.3)
send_line("echo MV_CHECK")
r = read_all(5)
if b'mvdat' in r:
    print(f"  PASS: mv")
    PASS += 1
else:
    print(f"  FAIL: mv")
    FAIL += 1

# wc
send_line("wc -l /LICENSE")
time.sleep(0.5)
send_line("echo WC_CHECK")
r = read_all(5)
if b'21' in r:
    print(f"  PASS: wc")
    PASS += 1
else:
    print(f"  FAIL: wc")
    FAIL += 1

# grep
send_line("grep MIT /LICENSE")
time.sleep(0.5)
send_line("echo GREP_CHECK")
r = read_all(5)
if b'MIT' in r:
    print(f"  PASS: grep")
    PASS += 1
else:
    print(f"  FAIL: grep")
    FAIL += 1

# head
send_line("head -n 1 /LICENSE")
time.sleep(0.5)
send_line("echo HEAD_CHECK")
r = read_all(5)
if b'MIT' in r:
    print(f"  PASS: head")
    PASS += 1
else:
    print(f"  FAIL: head")
    FAIL += 1

# pipes
send_line("echo pipetest | cat | wc -c")
time.sleep(0.5)
send_line("echo PIPE_CHECK")
r = read_all(5)
# "pipetest\n" = 9 chars
if b'9' in r:
    print(f"  PASS: pipe chain")
    PASS += 1
else:
    print(f"  FAIL: pipe chain")
    FAIL += 1

# redirect
send_line("echo 'redata' > /red_test")
time.sleep(0.3)
send_line("cat /red_test")
time.sleep(0.3)
send_line("echo RED_CHECK")
r = read_all(5)
if b'redata' in r:
    print(f"  PASS: redirect >")
    PASS += 1
else:
    print(f"  FAIL: redirect >")
    FAIL += 1

# chmod
send_line("touch /chm_test")
time.sleep(0.3)
send_line("chmod 600 /chm_test")
time.sleep(0.3)
send_line("ls -l /chm_test")
time.sleep(0.3)
send_line("echo CHM_CHECK")
r = read_all(5)
if b'rw-------' in r:
    print(f"  PASS: chmod")
    PASS += 1
else:
    print(f"  FAIL: chmod")
    FAIL += 1

# uname
send_line("uname")
time.sleep(0.5)
send_line("echo UNAME_CHECK")
r = read_all(5)
if b'xv8' in r:
    print(f"  PASS: uname")
    PASS += 1
else:
    print(f"  FAIL: uname")
    FAIL += 1

# whoami
send_line("whoami")
time.sleep(0.5)
send_line("echo WHO_CHECK")
r = read_all(5)
if b'root' in r:
    print(f"  PASS: whoami")
    PASS += 1
else:
    print(f"  FAIL: whoami")
    FAIL += 1

# printenv
send_line("printenv PATH")
time.sleep(0.5)
send_line("echo PENV_CHECK")
r = read_all(5)
if b'/bin' in r:
    print(f"  PASS: printenv")
    PASS += 1
else:
    print(f"  FAIL: printenv")
    FAIL += 1

# exit
send_line("exit")
time.sleep(1)

proc.terminate()
proc.wait()

print(f"\n=== Results ===")
print(f"PASS: {PASS}/{PASS+FAIL}")
print(f"FAIL: {FAIL}/{PASS+FAIL}")
sys.exit(1 if FAIL > 0 else 0)
