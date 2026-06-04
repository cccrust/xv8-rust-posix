# Memory Log for xv8-rust-posix

## Current State (v2.3 — completed)

xv8 QEMU integration tests pass 13/13 (added `_http`). Root `test.sh` suite passes 9/9 phases.

### Architecture

- `xv8rust/xv8-net/` — `#![no_std]` `std::net` compatibility layer for riscv64 (492 lines)
- `net/libnet/src/net_impl.rs` — platform abstraction: host uses `std::net`, xv8 uses `xv8_net::net`
- 9 net tools cross-compiled for riscv64: `dns`, `host`, `ntp`, `tcpclient`, `tcpserver`, `tftp`, `whois`, **`httpget`**, **`httpd`**
- HTTP tools (`httpget`/`httpd`) use raw `TcpStream` via `libnet`, no external HTTP crates
- `xv8/mkfs.sh` builds net tools + includes them in `fs.img`

### Key Test Infrastructure

- `xv8/user/testbin/nettools.rs` — QEMU testbin: fork+exec tcpserver + tcpclient, verifies TCP communication
- `xv8/user/testbin/http.rs` — QEMU testbin: fork+exec httpd + httpget, verifies HTTP request/response
- Root `test.sh` — comprehensive multi-phase test runner (stashes/restores posix/.cargo/config.toml)
- QEMU tests: fs, pipe, proc, fd, sbrk, cow, net, syscall, neteth, netdns, tcpecho, nettools, **http**

### Critical Gotchas

- kernel `tcp_connect()` / `tcp_accept()` are infinite busy-wait with no timeout — `kill()` cannot interrupt them
- xv8 user stack is only `USERSTACK = 4` pages (16 KB) — stack arrays > 16KB cause page fault
- `set_read_timeout`/`set_write_timeout` are no-op stubs on xv8
- `posix/.cargo/config.toml` has `[build] target = "riscv64gc-unknown-none-elf"` — must stash before host build

### Build Commands

```bash
# Host build
cargo build --release --manifest-path posix/Cargo.toml
cargo build --release --manifest-path net/Cargo.toml

# RISC-V cross-compile
cargo build --release --manifest-path posix/Cargo.toml --target riscv64gc-unknown-none-elf --no-default-features -Zbuild-std=core,alloc
cargo build --release --manifest-path net/Cargo.toml --target riscv64gc-unknown-none-elf --no-default-features --features xv8 -Zbuild-std=core,alloc

# Full suite
./test.sh
```

### Next Up

- v2.4: xv8rust async runtime + axum smoke test (xv8 target)
