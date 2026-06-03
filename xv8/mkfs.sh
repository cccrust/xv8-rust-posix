#!/bin/bash
set -e

cargo build --release --package user
cargo build --release --manifest-path ../posix/Cargo.toml --package tools --no-default-features
rm -f target/fs.img

# shellcheck disable=SC2046
posix_bins=$(find ../posix/target/riscv64gc-unknown-none-elf/release -maxdepth 1 -type f -perm -u+x | sort)

# Add only necessary xv8 binaries that don't conflict with posix
# init, poweroff, demo, primes, udp, uptime, zombie are unique to xv8
# We use full path so mkfs adds them with correct names (/init, /poweroff, etc.)
user_bins="
target/riscv64gc-unknown-none-elf/release/demo
target/riscv64gc-unknown-none-elf/release/dns
target/riscv64gc-unknown-none-elf/release/init
target/riscv64gc-unknown-none-elf/release/poweroff
target/riscv64gc-unknown-none-elf/release/primes
target/riscv64gc-unknown-none-elf/release/udp
target/riscv64gc-unknown-none-elf/release/uptime
target/riscv64gc-unknown-none-elf/release/zombie
"

# shellcheck disable=SC2086
cargo run \
  --release \
  --manifest-path mkfs/Cargo.toml \
  --target "$(rustc -vV | grep host | cut -d' ' -f2)" -- \
  target/fs.img \
  $posix_bins \
  $user_bins \
  LICENSE \
  "$@"
