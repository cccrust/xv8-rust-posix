# xv8-rust-posix Workspace

This is a Rust workspace containing two main projects:

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

# Run tests
./test.sh
```

**Key Features:**
- Boot: Entry at 0x80000000, machine-mode start, supervisor-mode init
- Memory: Buddy allocator, Sv39 paging, lazy allocation, COW fork
- Processes: 64 process slots, round-robin scheduler, sleep/wakeup
- Syscalls: fork, exec, wait, exit, open, read, write, pipe, socket, etc.
- Filesystem: Log-structured with write-ahead logging, inode-based
- Networking: Ethernet, ARP, IPv4, UDP, DHCP, loopback
- VirtIO: Block device (disk) and network drivers

**Kernel Crates:**
- `xv8` - Kernel crate (package name)
- `user` - User space library crate
- `mkfs` - Filesystem image creator

**Tests:** 7 tests (fs, pipe, proc, fd, sbrk, cow, net)

---

### posix/ - POSIX Utilities in Rust
A collection of POSIX-compliant command-line tools implemented in Rust.

**Location:** `posix/`

**Structure:**
```
posix/
├── Cargo.toml          # Workspace manifest
├── libposix/           # Shared library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── io.rs       # Read/Write traits
│       ├── fmt.rs
│       └── opt.rs
├── tools/              # Command-line utilities
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── bin/        # 100+ binary tools
│       └── tests/
└── _doc/              # Documentation (v0.1 - v0.19)
```

**Crate Names:**
- `libposix` - Shared library
- `tools` - Command utilities

**Build:**
```bash
cd posix
cargo build --release
```

**Notable Tools:**
- Phase 1: echo, true, false, yes, cat, wc, basename, dirname, sleep, kill, uname, printenv, env, whoami, id, hostname
- Phase 2: ls, cp, mv, rm, mkdir, rmdir, ln, touch, chmod, chown
- Phase 3: head, tail, sort, uniq, cut, tr, tee, od, cmp, diff
- Phase 4: grep, sed, xargs
- Phase 5: ps, du, df, stty, test, nice, nohup, date
- Phase 6: sh (shell)
- Phase 7: find, tar, patch, comm, fold, fmt, nl, expand, unexpand
- Advanced: ed, awk, bc, m4, pax, vi, make, man, lp, mailx, su, and many more

**Documentation:** See `posix/_doc/` for version history (v0.1 through v0.19)

---

### xv8_std/ - std library for xv8
Enable `std` support on xv8 so `posix/tools/` compiles without modification.

**Location:** `xv8_std/`

**Structure:**
```
xv8_std/
├── Cargo.toml
├── xv8-libc/           # syscall wrapper (C-like API)
└── xv8-std-overlay/    # std trait implementations
```

**Plan:** `xv8_std/v0.1.md` — Minimal impl for text-processing tools (cat, head, tail, grep, etc.)

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

**Note:** This is a multi-project workspace. Each subproject has its own AGENTS.md for detailed information.