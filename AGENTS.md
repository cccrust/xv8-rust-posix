# xv8-rust-posix Workspace

This is a Rust workspace containing three main projects:

## Projects

### xv8/ - RISC-V Operating System in Rust
Unix-like OS for RISC-V, inspired by xv6 and derived from octopus.

**Location:** `xv8/`

**Build & Run:**
```bash
cd xv8
cargo build --release
./mkfs.sh
cargo run --release
```

**Run tests:** `./test.sh`

### posix/ - POSIX Utilities in Rust
A collection of POSIX-compliant command-line tools implemented in Rust.

**Location:** `posix/`

**Build:** `cargo build --release`

### xv8_std/ - std library for xv8
Enable `std` support on xv8 so `posix/tools/` compiles without modification.

**Location:** `xv8_std/`

**Build:**
```bash
cd xv8_std/xv8-libc && cargo build --release
cd xv8_std/xv8-std-overlay && cargo build --release
```

---

## Workspace Root
**Files:**
- `README.md` - Project overview
- `LICENSE` - MIT license
- `.gitignore` - Git ignore rules
- `shell.sh` - Launch POSIX shell with network tools (see below)

**Note:** This is a multi-project workspace. Each subproject has its own AGENTS.md for detailed information.

### Using the POSIX Shell with Network Tools
To run the POSIX shell and have access to both the standard POSIX tools and the network tools (ping, host, etc.):
```bash
./shell.sh
```
This builds the posix/tools and net/tools workspaces, adds their release binaries to PATH, and executes the POSIX shell.
The shell prompt will be `posix> ` to distinguish it from the system shell.