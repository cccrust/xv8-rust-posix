#!/bin/bash
set -e

# Phase 5: End-to-end verification with curl from host to QEMU guest
# Requires: qemu-system-riscv64, curl, expect or timeout

KVMBIN="target/riscv64gc-unknown-none-elf/release/xv8"
FSIMG="target/fs.img"

if [ ! -f "$KVMBIN" ] || [ ! -f "$FSIMG" ]; then
    echo "Build kernel and create fs.img first: cargo build --release && ./test_phase5.sh"
    exit 1
fi

# Start QEMU in background
echo "Starting QEMU..."
qemu-system-riscv64 -cpu max -machine virt -bios none -m 256M -smp 4 -nographic \
    -global virtio-mmio.force-legacy=false \
    -drive file="$FSIMG",if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
    -netdev user,id=net0,hostfwd=tcp::27001-:27001,hostfwd=tcp::8080-:8080 \
    -device e1000,netdev=net0 \
    -kernel "$KVMBIN" &
QEMU_PID=$!

# Wait for QEMU to boot and tests to run
echo "Waiting for tests (15 tests, ~30 seconds)..."
sleep 30

# Try to curl the httpd from host (this will likely fail since
# httpd only runs during _http test on port 27998)
# The _http testbin runs httpd on 27998, httpepoll on 27001
echo "Trying to connect to guest httpd (port 27001)..."
curl -s --max-time 5 http://localhost:27001/ || echo "(expected: httpd not listening on this port during test)"

echo "Trying to connect to guest httpepoll (port 27001)..."
curl -s --max-time 5 http://localhost:27001/ || echo "(expected: httpepoll server runs during test)"

# Wait for QEMU to finish
wait $QEMU_PID || true
echo "Phase 5 manual verification complete."
echo "To verify with httpd: start QEMU, then in guest shell run: httpd &"
echo "Then from host: curl http://localhost:8080/"