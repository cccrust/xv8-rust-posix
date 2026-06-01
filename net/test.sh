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

echo ""
echo "=== net: NTP test ==="
OUT=$(cargo run --bin ntp pool.ntp.org 2>&1)
if echo "$OUT" | grep -q "Stratum"; then
    pass "ntp pool.ntp.org"
else
    fail "ntp pool.ntp.org"
fi

echo ""
echo "=== net: WHOIS test ==="
OUT=$(cargo run --bin whois google.com 2>&1)
if echo "$OUT" | grep -q "Domain Name"; then
    pass "whois google.com"
else
    fail "whois google.com"
fi

echo ""
echo "=== net: TCP echo test ==="
cargo run --bin tcpserver 19991 --echo > /dev/null 2>&1 &
SERVER_PID=$!
sleep 0.3
OUT=$(cargo run --bin tcpclient 127.0.0.1 19991 "hello" 2>&1 || true)
kill $SERVER_PID 2>/dev/null || true
if echo "$OUT" | grep -q "Received 5 bytes"; then
    pass "tcpclient echo"
else
    fail "tcpclient echo"
fi

echo ""
echo "=== net: TCP daytime test ==="
cargo run --bin tcpserver 19992 --daytime > /dev/null 2>&1 &
SERVER_PID=$!
sleep 0.3
OUT=$(cargo run --bin tcpclient 127.0.0.1 19992 "hello" 2>&1 || true)
kill $SERVER_PID 2>/dev/null || true
if echo "$OUT" | grep -q "T"; then
    pass "tcpclient daytime"
else
    fail "tcpclient daytime"
fi

echo ""
echo "=== net: TCP time test ==="
cargo run --bin tcpserver 19993 --time > /dev/null 2>&1 &
SERVER_PID=$!
sleep 0.3
OUT=$(cargo run --bin tcpclient 127.0.0.1 19993 "hello" 2>&1 || true)
kill $SERVER_PID 2>/dev/null || true
if echo "$OUT" | grep -qE "[0-9]{7,}"; then
    pass "tcpclient time"
else
    fail "tcpclient time"
fi

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [ $FAIL -gt 0 ]; then
    exit 1
fi
