# Memory Log for xv8-rust-posix

## Current State (v2.4 — completed)

xv8 QEMU integration tests pass **14/14** (added `_async`). Root `test.sh` suite passes **9/9** phases.

### v2.4 Accomplishment: Async Runtime on xv8 QEMU

Created a minimal async executor testbin (`_async`) that runs on xv8 demonstrating async/await with `block_on` and `Sleep`:

- **Self-contained, zero-external-deps** — uses only `core::future`, no `alloc`, no `Arc`, no atomics
- **No privileged CSR instructions** — avoided `csrr sstatus` trap by eliminating all `core::sync::atomic` usage (compiler generates `csrr sstatus` for `riscv64gc-unknown-none-elf` on `compare_exchange`/`Arc`/`Weak`)
- **Dummy waker** — spin-poll loop with `nanosleep` between iterations (safe for single-threaded xv8)
- **`Sleep` future** — polls uptime until deadline, returns `Pending`/`Ready`
- **6 tests** pass: `block_on value`, `block_on expr`, `sleep 10ms`, `two sleeps`, `loop+sleep`, `nested block_on`

### Key Technical Insight

`riscv64gc-unknown-none-elf` target generates `csrr sstatus` / `csrw sstatus` for ALL `core::sync::atomic` operations (including `Arc`, `Weak`, `AtomicBool`, `AtomicUsize`). This traps in user mode as illegal instruction. Solution: use `Pin<&mut F>` on stack + dummy waker + poll-loop — zero atomics, zero CSR access.

### Architecture

- `xv8/user/testbin/async.rs` — self-contained async runtime testbin (block_on + Sleep)
- Already integrated: `xv8/user/testbin/testrunner.rs` lists `/_async`, `xv8/user/Cargo.toml` has `_async` bin

### Previous (v2.3)

- `xv8rust/xv8-net/` — `#![no_std]` `std::net` compatibility layer for riscv64 (492 lines)
- `net/libnet/src/net_impl.rs` — platform abstraction: host uses `std::net`, xv8 uses `xv8_net::net`
- 9 net tools cross-compiled for riscv64: `dns`, `host`, `ntp`, `tcpclient`, `tcpserver`, `tftp`, `whois`, `httpget`, `httpd`
- HTTP tools use raw `TcpStream` via `libnet`, no external HTTP crates
- `xv8/mkfs.sh` builds net tools + includes them in `fs.img`
- Testbins: fs, pipe, proc, fd, sbrk, cow, net, syscall, neteth, netdns, tcpecho, nettools, http

### Critical Gotchas

- kernel `tcp_connect()` / `tcp_accept()` are infinite busy-wait with no timeout — `kill()` cannot interrupt them
- xv8 user stack is only `USERSTACK = 4` pages (16 KB) — stack arrays > 16KB cause page fault
- `set_read_timeout`/`set_write_timeout` are no-op stubs on xv8
- `posix/.cargo/config.toml` has `[build] target = "riscv64gc-unknown-none-elf"` — must stash before host build
- `riscv64gc-unknown-none-elf` target generates `csrr sstatus` for all atomic/Arc operations — traps in user mode

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

- v2.5: Richer async runtime (`xv8-async` crate) with proper waker-based scheduling, task spawning, timer reactor
- xv8-axum-smoke: integration test with tokio+axum patched for xv8 target
