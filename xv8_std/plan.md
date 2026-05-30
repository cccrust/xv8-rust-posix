# xv8_std Architecture Plan

## Goal
Enable `std` support on xv8 so `posix/tools/` compiles without modification on xv8, Linux, and macOS.

## Architecture

```
posix/tools/          ← needs std traits for user types
    ↓
xv8-user-std/         ← implements std traits for user types (foreign impl)
    ↓
user/                 ← provides Fd, Args, syscall wrappers (target-specific)
    ↓
kernel/               ← xv8 kernel (no_std)
```

## Crates

### xv8-libc
Thin syscall wrappers. Re-exports `user::syscall::raw::*`.

**Current issue:** Depends on `user` crate which is a HOST crate (uses std, edition 2024). Cannot be compiled for `riscv64gc-unknown-none-elf` until `user` is `no_std`.

**Status:** Code written, blocked on `user` being `no_std`.

### xv8-user-std
Implements `std` traits for `user` types (`Fd`, `Args`).

**Dependencies:**
- `user` (from xv8 workspace)
- `alloc` (for `no_std` target via `build-std`)

**Status:** Code written, needs `user` to be `no_std` and `build-std=alloc` configured.

## Build Configuration

### For host (Linux/macOS)
- Build with standard Cargo (uses system `std`)
- `user` crate compiled for host

### For xv8 target (`riscv64gc-unknown-none-elf`)
- Requires `user` crate to be `no_std`
- Requires `-Z build-std=alloc,core` for `alloc` support
- All crates in dependency chain must be built with same flags

## Dependency Chain for xv8 target

```
posix/tools
  → libposix
  → xv8-user-std  (std traits for user types)
  → user (Fd, Args, syscalls) [no_std]
  → kernel (xv8) [no_std]
  → alloc [from build-std source]
```

## Key Files

| File | Purpose |
|------|---------|
| `xv8_std/xv8-libc/src/lib.rs` | Syscall wrappers |
| `xv8_std/xv8-user-std/src/io.rs` | std::io traits |
| `xv8_std/xv8-user-std/src/fs.rs` | std::fs types |
| `xv8_std/xv8-user-std/src/path.rs` | std::path types |
| `xv8_std/xv8-user-std/src/env.rs` | std::env |
| `xv8_std/xv8-user-std/src/process.rs` | std::process (stub) |
| `xv8/user/src/syscall.rs` | Raw syscalls (added lseek) |

## Blockers

### 1. `user` crate is not `no_std`
- Currently uses `std` and edition 2024
- Must be converted to `no_std` for xv8 target build
- This is a significant refactoring task

### 2. `alloc` for bare-metal target
- Must be compiled from source via `-Z build-std=alloc`
- Requires ALL crates in dependency chain to use same `core`/`alloc`
- Requires clean build of everything

### 3. Workspace configuration
- `user` is in `xv8` workspace
- `posix/tools` is in `posix` workspace
- Cross-workspace dependencies require careful path management

## Alternative Approaches

### Option A: Make `user` crate `no_std` compatible
- Add `#![no_std]` to `user/src/lib.rs`
- Replace `std` usage with `core`/`alloc`
- Then `xv8-libc` and `xv8-user-std` can be compiled for xv8 target

### Option B: Restructure posix/tools into xv8 workspace
- Make `posix/tools` a member of `xv8` workspace
- Use `build-std` to replace `std` with `xv8-user-std`
- Simplifies dependency management but modifies project structure

### Option C: Host-only build first
- Focus on making `posix/tools` work on Linux/macOS with `xv8-libc` as syscall bridge
- Defer xv8 target to later phase
- Much simpler since `user` is already a host-compatible crate

## Decision

**Recommended: Option C (Host-only first)**
- Get `posix/tools` working on Linux/macOS first using `xv8-libc` as the syscall interface
- This validates the trait implementations without dealing with `no_std` complexity
- xv8 target can be addressed later once host works

**Then Option A** when ready for xv8 target.

## History

### 2024-xx-xx - Initial implementation
- Created `xv8-libc` and `xv8-user-std` crates
- Implemented `io.rs`, `fs.rs`, `path.rs`, `env.rs`, `process.rs`
- Added `lseek` syscall to `user/src/syscall.rs`
- Discovered `user` crate is not `no_std`, blocking xv8 target build