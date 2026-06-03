# xv8-rust-posix Workspace

Multi-project Rust workspace (NOT a root Cargo workspace). Four independent workspaces.

| Project | Dir | Type |
|---------|-----|------|
| xv8 OS | `xv8/` | RISC-V Unix-like OS (nightly + `qemu-system-riscv64`) |
| POSIX tools | `posix/` | 124 POSIX utilities + shell |
| xv8 std | `xv8_std/` | std overlay so POSIX tools compile for riscv64 |
| Network tools | `net/` | ping, dns, tcp, echo, ntp, whois, http, curl, ssh |

## Key Commands

```bash
./shell.sh                        # Build + launch POSIX shell with net tools (prompt: posix>)
./test.sh                         # Full suite: posix tests + xv8 cross-compile + QEMU integration
cd posix && ./test.sh             # Shell (33/33) + core tools (21/21) tests via shell scripts
cd xv8   && ./test.sh             # 10 QEMU integration tests (builds kernel+user, creates fs.img)
cd net   && ./test.sh             # Network smoke (dns, ntp, tcp echo, whois)
./test_posix_host.sh              # Phase 1-5 individual tool tests (macOS, bypasses build-std)
./test_posix_all_host.sh          # Comprehensive smoke test of all 124 tools
```

## Architecture Gotchas

- **No root Cargo.toml** — each subproject (`xv8/`, `posix/`, `xv8_std/`, `net/`) is its own workspace
- **`.cargo/rustc-wrapper.sh`** (root `[build]` config) — injects `#![no_main]` + `#[no_mangle]` on `src/bin/*.rs` when targeting riscv64. This is what lets POSIX tools compile unchanged for both host and RISC-V.
- **`posix/.cargo/config.toml`** — riscv64 cross-compile config (target, linker, `build-std`). **`test_posix_host.sh` stashes this to `.bak`** to avoid host build conflicts, then restores it. If host builds fail, check if this file is missing.
- **`libc` → `xv8-libc-compat`** — `posix/tools/Cargo.toml` imports `libc` which resolves to `xv8_std/xv8-libc-compat`. On riscv64 provides syscall wrappers; on host delegates to real `libc`.
- **`crossterm` → `xv8_std/crossterm/`** — vendored crossterm 0.29.0. Enables vi/vim on host. riscv64 uses `--no-default-features` to exclude it (`vi`/`vim` have `required-features = ["crossterm"]`).
- **`riscv64gc-unknown-none-elf` is neither `cfg(unix)` nor `cfg(windows)`** — platform-specific code must check `target_arch`.
- **Toolchain**: nightly + target `riscv64gc-unknown-none-elf` (set in `xv8/rust-toolchain.toml`)
- **Resolver mismatch**: `net/` uses resolver `"2"`, `posix/` and `xv8_std/` use `"3"`
- **xv8 release profile**: `lto = true`, `strip = true`, `codegen-units = 1`
- **`posix/AGENTS.md` mentions `libposix/` directory — it does not exist**. The workspace has members `["tools"]` only. `tools/src/lib.rs` is the shared library.

## Cross-Compilation

```bash
cargo build --release --manifest-path posix/Cargo.toml --target riscv64gc-unknown-none-elf --no-default-features
cargo build --release --manifest-path xv8_std/Cargo.toml --target riscv64gc-unknown-none-elf
cargo build --release -p tools --target riscv64gc-unknown-none-elf --features crossterm  # for vi/vim
```

## POSIX Test Details

Tests are shell scripts in `posix/tools/tests/`, run via:
```bash
cd posix && cargo build --release && PATH="target/release:$PATH" sh tools/tests/test_sh_basic.sh
```
- `test_sh_basic.sh` — 33 shell tests (pipes, redirects, vars, control flow)
- `test_tools_core.sh` — 21 core tool tests (echo, cat, wc, basename, etc.)
- `test_v2.0.sh` — v2.0 tools (rev, col, look, who, users, last)
- `basic.rs` — Rust integration test (run via `cargo test`)

## xv8 QEMU Test Flow

`xv8/test.sh`:
1. Build user programs (`cargo build --release --package user`)
2. Create a fresh 256M `fs.img` with test binaries + `/tmp/testmode` marker
3. Run QEMU (`cargo run --release`)
4. init.rs detects testmode, runs testrunner, which executes 10 tests
5. Restores original `fs.img` from backup on exit

## net/ Cross-Platform Design

`net/libnet/` uses `std::net` on host (linux/mac). For xv8 target, the kernel provides its own net stack (Ethernet, ARP, IPv4, UDP, ICMP, DHCP, TCP) with syscall wrappers. `xv8_std/xv8-libc-compat` bridges the gap. TCP is fully functional on loopback (v2.1, 11/11 QEMU tests pass).

## Planning & Docs

- `_doc/todo.md` — master development plan (v1.3+)
- `_doc/xv8-net-status.md` — detailed network stack architecture
- `MEMORY.md` — session notes (may be stale)
- `_doc/v*.md` — per-version changelogs