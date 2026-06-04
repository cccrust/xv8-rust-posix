# xv8-rust-posix

Multi-project Rust workspace (NOT a root Cargo.toml). Four independent workspaces.

| Project | Dir | Type |
|---------|-----|------|
| xv8 OS | `xv8/` | RISC-V Unix-like OS (nightly + `qemu-system-riscv64`) |
| POSIX tools | `posix/` | 124 POSIX utilities + shell |
| xv8 std | `xv8rust/` | std overlay + experimental async/runtime for riscv64 |
| Network tools | `net/` | ping, dns, tcp, echo, ntp, whois, http, curl, ssh |

## Commands

```bash
./shell.sh                        # Build + launch POSIX shell with net tools (prompt: posix>)
./test.sh                         # posix cargo test → cross-compile xv8 → QEMU integration
cd posix && ./test.sh             # Shell (33) + core tools (21)
cd xv8   && ./test.sh             # 10 QEMU integration tests
cd net   && ./test.sh             # dns, ntp, tcp echo, whois
./test_posix_host.sh              # Phase 1-5 host tests (stashes posix/.cargo/config.toml)
./test_posix_all_host.sh          # All 124 tools smoke test on macOS
./test_crossterm.sh               # riscv64 crossterm + host test
```

## Architecture Gotchas

- **No root Cargo.toml** — each subproject (`xv8/`, `posix/`, `xv8rust/`, `net/`) is its own workspace
- **`.cargo/rustc-wrapper.sh`** (root `[build]` config) — injects `#![no_main]` + `#[no_mangle]` on `src/bin/*.rs` for riscv64. Enables POSIX tools to compile for both host and RISC-V.
- **`posix/.cargo/config.toml`** sets `build.target = riscv64gc-unknown-none-elf` — plain `cargo build` in `posix/` cross-compiles. `test_posix_host.sh` stashes to `.bak` for host builds, restores after. If host builds fail, this file is likely still missing.
- **`libc` → `xv8-libc-compat`** — `posix/tools/Cargo.toml` imports `libc` resolving to `xv8rust/xv8-libc-compat` (syscall wrappers on riscv64, delegates to real libc on host)
- **`crossterm` → `xv8rust/crossterm/`** — vendored crossterm 0.29.0. `vi`/`vim` have `required-features = ["crossterm"]`; riscv64 build uses `--no-default-features` to exclude it
- **`riscv64gc-unknown-none-elf` is neither `cfg(unix)` nor `cfg(windows)`** — platform-specific code checks `target_arch`
- **Toolchain**: nightly + `riscv64gc-unknown-none-elf` (`xv8/rust-toolchain.toml`)
- **Resolver mismatch**: `net/` uses resolver `"2"`, `posix/` and `xv8rust/` use `"3"`
- **Release profiles**: `lto = true`, `strip = true`, `codegen-units = 1` in both `xv8/` and `posix/`
- **`xv8rust/Cargo.toml`** patches `tokio` to `/Users/Shared/ccc/github/tokio/tokio` — local absolute path, only exists on dev machine
- **`posix/AGENTS.md` mentions `-p libposix`** — no such crate; workspace has `members = ["tools"]` only, shared code in `tools/src/lib.rs`
- **`net/libnet/`** uses `std::net` on host. On riscv64, the kernel provides its own net stack (Ethernet → TCP) with syscall wrappers via `xv8-libc-compat`.

## Cross-Compile

```bash
cargo build --release --manifest-path posix/Cargo.toml --target riscv64gc-unknown-none-elf --no-default-features
cargo build --release --manifest-path xv8rust/Cargo.toml --target riscv64gc-unknown-none-elf
cargo build --release -p tools --target riscv64gc-unknown-none-elf --features crossterm  # vi/vim only
```

## Testing

- **Shell scripts**: `posix/tools/tests/test_sh_basic.sh` (33), `test_tools_core.sh` (21), `test_v2.0.sh`
- **Rust test**: `cargo test --release -p tools` (`tests/basic.rs`)
- **Run single**: `cd posix && cargo build --release && PATH="target/release:$PATH" sh tools/tests/test_sh_basic.sh`
- **xv8 QEMU flow** (`xv8/test.sh`): builds user progs → creates 256M `fs.img` with `/tmp/testmode` → runs QEMU → init.rs detects testmode, runs testrunner (10 tests) → restores `fs.img`
- **`mkfs.sh`** builds posix tools for riscv64 (`--no-default-features`) and embeds them in `fs.img`
