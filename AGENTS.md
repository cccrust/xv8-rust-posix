# xv8-rust-posix Workspace

Multi-project Rust workspace (NOT a Cargo workspace at root level). Four subprojects in separate directories.

## Subprojects

| Project | Dir | Type |
|---------|-----|------|
| xv8 OS | `xv8/` | RISC-V Unix-like OS (requires nightly + `qemu-system-riscv64`) |
| POSIX tools | `posix/` | 100+ POSIX utilities + shell |
| xv8 std | `xv8_std/` | std overlay so POSIX tools compile for riscv64 |
| Network tools | `net/` | ping, dns, tcp, etc. |

## Key Commands

```bash
./shell.sh              # Build + launch POSIX shell with net tools (prompt: posix> )
./test.sh               # Full suite: posix tests + xv8 cross-compile + QEMU integration
cd xv8   && ./test.sh   # QEMU integration tests only (10 tests, requires qemu-system-riscv64)
cd posix && ./test.sh   # POSIX shell (33/33) + core tools (21/21) tests
cd net   && ./test.sh   # Network tools smoke tests
```

## Critical Architecture Notes

- **No root Cargo.toml** — each subproject is its own workspace
- **`.cargo/rustc-wrapper.sh`** — injects `#![no_main]` and `#[no_mangle]` for `src/bin/*.rs` when targeting `riscv64gc-unknown-none-elf`. This is what allows POSIX tools to compile for both host and RISC-V without source changes.
- **`libc` → `xv8-libc-compat`** — `posix/tools/Cargo.toml` depends on `libc` which resolves to `xv8_std/xv8-libc-compat`. On `riscv64` this provides minimal syscall wrappers; on host it delegates to real `libc`.
- **Toolchain**: nightly + target `riscv64gc-unknown-none-elf`
- **QEMU**: qemu-system-riscv64, virt machine, 256M, 4 cores, E1000 NIC, virtio-blk

## xv8 Build Flow

```bash
cargo build --release   # kernel + user programs for riscv64
./mkfs.sh              # creates fs.img with user binaries
cargo run --release    # runs in QEMU
```

## Cross-Compilation Gotcha

Always use `--target riscv64gc-unknown-none-elf` for riscv64 builds:
```bash
cargo build --release --manifest-path posix/Cargo.toml --target riscv64gc-unknown-none-elf
cargo build --release --manifest-path xv8_std/Cargo.toml --target riscv64gc-unknown-none-elf
```

## Existing Instruction Files

Each subproject has its own `AGENTS.md` with detailed information.