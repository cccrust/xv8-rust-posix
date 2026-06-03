# xv8-rust-posix Workspace

Multi-project Rust workspace (NOT a root Cargo workspace). Four subprojects.

| Project | Dir | Type |
|---------|-----|------|
| xv8 OS | `xv8/` | RISC-V Unix-like OS (nightly + `qemu-system-riscv64`) |
| POSIX tools | `posix/` | 100+ POSIX utilities + shell |
| xv8 std | `xv8_std/` | std overlay so POSIX tools compile for riscv64 |
| Network tools | `net/` | ping, dns, tcp, echo, ntp, whois |

## Key Commands

```bash
./shell.sh              # Build + launch POSIX shell with net tools (prompt: posix>)
./test.sh               # Full suite: posix tests + xv8 cross-compile + QEMU integration
cd posix && ./test.sh   # POSIX shell (33/33) + core tools (21/21) tests
cd xv8   && ./test.sh   # QEMU integration tests, 10 tests
cd net   && ./test.sh   # Network smoke tests (dns, ntp, tcp echo, whois)
```

Additional scripts for host-only testing:
```bash
./test_posix_host.sh      # Phase 1-5 individual tool tests for macOS
./test_posix_all_host.sh  # Comprehensive smoke test of all 124 tools
```

## Architecture Gotchas

- **No root Cargo.toml** — each subproject is its own workspace
- **`.cargo/rustc-wrapper.sh`** (root) — injects `#![no_main]` + `#[no_mangle]` on `main` for `src/bin/*.rs` when targeting `riscv64gc-unknown-none-elf`. This is what lets POSIX tools compile unchanged for both host and RISC-V.
- **posix/.cargo/config.toml.bak** — the riscv64 cross-compile config for posix (target, linker, `build-std`). It was stashed by `test_posix_host.sh`; rename back to `config.toml` when building for riscv64.
- **`libc` → `xv8-libc-compat`** — `posix/tools/Cargo.toml` imports `libc` which resolves to `xv8_std/xv8-libc-compat`. On riscv64 provides syscall wrappers; on host delegates to real `libc`.
- **`crossterm` → `xv8_std/crossterm/`** — vendored crossterm 0.29.0. Enables vi/vim on host. riscv64 uses `--no-default-features` to exclude it (`vi`/`vim` have `required-features = ["crossterm"]`).
- **`riscv64gc-unknown-none-elf` is neither `cfg(unix)` nor `cfg(windows)`** — platform-specific crossterm code excluded unless riscv64 backends added.
- **Toolchain**: nightly + target `riscv64gc-unknown-none-elf`
- **Resolver mismatch**: `net/` uses resolver `"2"`, `posix/` and `xv8_std/` use `"3"`
- **xv8 release profile**: `lto = true`, `strip = true`, `codegen-units = 1`

## Cross-Compilation

```bash
cargo build --release --manifest-path posix/Cargo.toml --target riscv64gc-unknown-none-elf --no-default-features
cargo build --release --manifest-path xv8_std/Cargo.toml --target riscv64gc-unknown-none-elf
```

## Master Plan

Root `_doc/todo.md` is the development plan (v1.3+). Subproject `_doc/` dirs may have stale info. `MEMORY.md` holds session notes.
