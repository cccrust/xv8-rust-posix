# xv8-rust-posix

Multi-project Rust workspace (NOT a root Cargo workspace). Four independent workspaces.

| Project | Dir | Type |
|---------|-----|------|
| xv8 OS | `xv8/` | RISC-V Unix-like OS (nightly + `qemu-system-riscv64`) |
| POSIX tools | `posix/` | 124 POSIX utilities + shell |
| xv8 std | `xv8rust/` | std overlay + experimental async/runtime + HTTP/router for riscv64 |
| Network tools | `net/` | ping, dns, tcp, echo, ntp, whois, http, curl, ssh |

## Commands

```bash
./shell.sh                        # Build + launch POSIX shell with net tools (prompt: posix>)
./test.sh                         # Full suite: posix tests + xv8 cross-compile + QEMU
cd posix && ./test.sh             # Shell (33/33) + core tools (21/21) tests
cd posix && sh tools/tests/test_v2.0.sh  # rev, col, look, who, users, last
cd xv8   && ./test.sh             # 14 QEMU integration tests (builds kernel+user, fs.img)
cd net   && ./test.sh             # Network smoke (dns, ntp, tcp echo, whois)
```

## Gotchas

- **No root Cargo.toml** — each subproject is its own workspace; always use `--manifest-path`
- **`.cargo/rustc-wrapper.sh`** (root `[build]` config) — injects `#![no_main]` + `#[no_mangle]` on `src/bin/*.rs` for riscv64, letting POSIX tools compile for both host and RISC-V unchanged
- **`posix/.cargo/config.toml`** sets `[build] target = "riscv64gc-unknown-none-elf"` with `build-std`. `./test.sh` stashes it to `.bak` for host builds, restores after. If host builds fail, check that this file isn't missing (the `.bak` from a failed run leaves the config stashed)
- **`libc` → `xv8-libc-compat`** — `posix/tools/Cargo.toml` imports `libc` which resolves to `xv8rust/xv8-libc-compat`. On riscv64 provides syscall wrappers; on host delegates to real `libc`
- **`crossterm` → `xv8rust/crossterm/`** — vendored crossterm 0.29.0. `vi`/`vim` have `required-features = ["crossterm"]`; riscv64 uses `--no-default-features` to exclude it
- **`riscv64gc-unknown-none-elf` is neither `cfg(unix)` nor `cfg(windows)`** — platform-specific code must check `target_arch`
- **Toolchain**: nightly + target `riscv64gc-unknown-none-elf` (set in `xv8/rust-toolchain.toml`)
- **Resolver mismatch**: `net/` uses resolver `"2"`, `posix/` and `xv8rust/` use `"3"`
- **`xv8rust/Cargo.toml` patches tokio** to a local path (`/Users/Shared/...`) — only works on the original dev machine
- **`posix/AGENTS.md` mentions `libposix/` — it does not exist**. Workspace has `members = ["tools"]` only
- **xv8 kernel TCP**: `tcp_connect()`/`tcp_accept()` are infinite busy-waits (no timeout); `kill()` cannot interrupt them
- **xv8 user stack**: only 4 pages (16 KB) — stack arrays > 16 KB cause page fault
- **CSR trap**: `riscv64gc-unknown-none-elf` generates `csrr sstatus` for all `core::sync::atomic` ops (Arc, AtomicBool, etc.) — this traps in user mode. Avoid atomics in riscv64 user code

## Cross-Compilation

```bash
# POSIX tools
cargo build --release --manifest-path posix/Cargo.toml --target riscv64gc-unknown-none-elf --no-default-features

# xv8 std
cargo build --release --manifest-path xv8rust/Cargo.toml --target riscv64gc-unknown-none-elf

# Net tools (needs explicit build-std)
cargo build --release --manifest-path net/Cargo.toml --package tools \
    --no-default-features --features xv8 -Zbuild-std=core,alloc --target riscv64gc-unknown-none-elf

# With crossterm (vi/vim)
cargo build --release -p tools --target riscv64gc-unknown-none-elf --features crossterm
```

## Testing

POSIX tests are shell scripts under `posix/tools/tests/`:
```bash
cd posix && cargo build --release && PATH="target/release:$PATH" sh tools/tests/test_sh_basic.sh
cd posix && cargo build --release && PATH="target/release:$PATH" sh tools/tests/test_tools_core.sh
cd posix && cargo build --release && PATH="target/release:$PATH" sh tools/tests/test_v2.0.sh
# Rust integration test: cargo test
# shtest (93 shell behavior tests):
cargo build --release -p tools --bin sh && target/release/sh xv8/shtest.sh
```

xv8 QEMU flow (`xv8/test.sh`):
1. Build user programs (`cargo build --release --package user`)
2. Build POSIX tools (`cargo build --manifest-path ../posix/Cargo.toml --package tools --no-default-features`)
3. Create fresh 256M `fs.img` with POSIX tools + xv8 test binaries + `/tmp/testmode` marker
4. Run QEMU (`cargo run --release`); `init.rs` detects testmode and runs testrunner
5. Restores original `fs.img` from backup on exit
6. 17 testbins across 4 categories: core (fs, pipe, proc, fd, sbrk, cow, syscall), net (net, neteth, netdns, tcpecho, nettools, http), async (async, httpepoll, axum), shell (shtest)

## xv8rust Crates

| Crate | Path | Description |
|-------|------|-------------|
| `xv8-libc` | `xv8rust/xv8-libc/` | 52 raw syscall wrappers (RISC-V inline asm) |
| `xv8-user-std` | `xv8rust/xv8-user-std/` | Userspace std overlay (io, net, sync, time, fs) |
| `xv8-async` | `xv8rust/xv8-async/` | Async runtime + epoll reactor |
| `xv8-tokio-compat` | `xv8rust/xv8-tokio-compat/` | Tokio-compatible traits (AsyncRead/Write, TcpStream, runtime) |
| `xv8-http` | `xv8rust/xv8-http/` | HTTP/1.1 types + parser (`#![no_std]`) |
| `xv8-router` | `xv8rust/xv8-router/` | Lightweight axum-like Router (`#![no_std]`) |

## Documentation

- `_doc/` — planning docs (v5.x changelogs)
- `_wiki/` — technical wiki (OS concepts, containers, POSIX, network)
- `README.md` per sub-project (posix/, xv8rust/, net/)
- `AGENTS.md` per sub-project
- `xv8/kernel/src/*.md` — inline kernel module docs (file, fs, proc, namespace, cgroup, overlay, seccomp, capability, eventfd, memfd, pidfd, signalfd, timerfd, inotify, poll, etc.)
- `posix/tools/src/bin/*.md` — inline tool docs

## Planning Docs

- `_doc/todo.md` — master development plan
- `_doc/xv8-net-status.md` — network stack architecture
- `_doc/v*.md` — per-version changelogs (v1.3–v2.9)
