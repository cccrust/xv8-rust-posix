#!/bin/bash
set -e

PASS=0
FAIL=0

pass() { PASS=$((PASS+1)); echo "PASS: $1"; }
fail() { FAIL=$((FAIL+1)); echo "FAIL: $1"; }

cleanup() {
    kill $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

echo "=== net: libnet unit tests ==="
cargo test -p libnet 2>&1

echo ""
echo "=== net: build ==="
cargo build 2>&1

echo ""
echo "=== net: dns smoke test ==="
OUT=$(cargo run --bin dns google.com 8.8.8.8 2>&1)
if echo "$OUT" | grep -q "A "; then
    pass "dns google.com"
else
    fail "dns google.com"
fi

echo ""
echo "=== net: host smoke test ==="
OUT=$(cargo run --bin host google.com 8.8.8.8 2>&1)
if echo "$OUT" | grep -q "has address"; then
    pass "host google.com"
else
    fail "host google.com"
fi

run_tcp_test() {
    local name=$1 mode=$2 port=$3 expect=$4
    cargo run --bin tcpserver $port $mode > /dev/null 2>&1 &
    local pid=$!
    sleep 0.3
    OUT=$(cargo run --bin tcpclient 127.0.0.1 $port "hello" 2>&1 || true)
    kill $pid 2>/dev/null || true
    if echo "$OUT" | grep -q "$expect"; then
        pass "tcpclient $name"
    else
        fail "tcpclient $name"
    fi
}

echo ""
echo "=== net: TCP echo test ==="
run_tcp_test "echo" "--echo" 19991 "Received 5 bytes"

echo ""
echo "=== net: TCP daytime test ==="
run_tcp_test "daytime" "--daytime" 19992 "T"

echo ""
echo "=== net: TCP time test ==="
run_tcp_test "time" "--time" 19993 "[0-9]"

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [ $FAIL -gt 0 ]; then
    exit 1
fi
