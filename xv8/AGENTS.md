# xv8 - RISC-V Operating System in Rust

xv8 is a Unix-like operating system for RISC-V processors, built entirely in Rust. It is inspired by xv6 and supports RISC-V hardware including multiple harts, virtual memory, filesystem, networking, and more.

## Project Structure

```
xv8/
├── Cargo.toml          # Workspace manifest
├── Cargo.lock
├── kernel/              # Kernel crate
│   ├── Cargo.toml
│   ├── build.rs
│   ├── kernel.ld        # Linker script
│   └── src/
│       ├── main.rs      # Binary entry point
│       ├── lib.rs       # Kernel library & main()
│       ├── abi.rs       # Syscall ABI definitions
│       ├── entry.rs     # Boot entry point
│       ├── start.rs     # Machine mode start
│       ├── param.rs     # Kernel parameters
│       ├── memlayout.rs # Memory layout constants
│       ├── console.rs   # Console/UART driver
│       ├── uart.rs      # UART hardware driver
│       ├── plic.rs      # PLIC interrupt controller
│       ├── kernelvec.rs # Kernel trap vector
│       ├── trap.rs      # Trap handling
│       ├── trampoline.rs # User/kernel transition
│       ├── riscv.rs     # RISC-V CSRs & utilities
│       ├── vm.rs        # Virtual memory (Sv39)
│       ├── kalloc.rs    # Physical memory allocator
│       ├── proc.rs      # Process management & scheduler
│       ├── swtch.rs     # Context switch
│       ├── spinlock.rs  # Spinlock implementation
│       ├── sleeplock.rs # Sleep lock (blocking)
│       ├── sync.rs      # Synchronization primitives
│       ├── printf.rs    # Printf implementation
│       ├── error.rs     # Error types
│       ├── buf.rs       # Buffer cache
│       ├── log.rs       # Write-ahead logging
│       ├── fs.rs        # Filesystem (inode, directory)
│       ├── file.rs      # File descriptor abstraction
│       ├── exec.rs      # ELF exec implementation
│       ├── pipe.rs      # Pipe implementation
│       ├── syscall.rs   # Syscall dispatcher
│       ├── sysproc.rs   # Process syscalls
│       ├── sysfile.rs   # File syscalls
│       ├── virtio_disk.rs # VirtIO block driver
│       ├── pci.rs       # PCI bus enumeration
│       ├── e1000.rs     # E1000 network driver
│       ├── rng.rs       # Random number generator
│       └── net/         # Network stack
│           ├── mod.rs
│           ├── eth.rs
│           ├── arp.rs
│           ├── ipv4.rs
│           ├── icmp.rs
│           ├── udp.rs
│           ├── dhcp.rs
│           ├── route.rs
│           ├── interface.rs
│           └── loopback.rs
├── user/                # User space programs
│   ├── Cargo.toml
│   ├── build.rs
│   ├── user.ld
│   ├── src/
│   │   ├── lib.rs
│   │   ├── syscall.rs   # Syscall wrappers
│   │   ├── io.rs        # I/O traits for Fd
│   │   ├── line.rs      # Line editor
│   │   └── args.rs      # Argument parsing
│   ├── bin/             # User binaries
│   │   ├── init.rs      # First userspace process
│   │   ├── sh.rs        # Shell
│   │   ├── cat.rs, ls.rs, echo.rs, etc.
│   │   ├── udp.rs       # UDP test utility
│   │   └── dns.rs       # DNS lookup tool
│   └── testbin/         # Internal test programs
│       ├── testrunner.rs
│       ├── fs.rs, pipe.rs, proc.rs, neteth.rs, netdns.rs, etc.
├── mkfs/                # Filesystem image creator
│   ├── Cargo.toml
│   └── src/main.rs
├── .cargo/config.toml  # QEMU runner config
├── rust-toolchain.toml  # Rust toolchain
├── mkfs.sh              # Create fs.img
├── setup_net.sh         # Setup tap network interface (legacy)
├── run.sh               # Run tests in QEMU
└── test.sh              # Run all tests in QEMU
```

## Build & Run

```bash
# Build kernel and user programs
cargo build --release

# Create filesystem image
./mkfs.sh

# Run in QEMU
cargo run --release

# Run tests
./test.sh
```

## Key Features

- **Boot**: Entry at 0x80000000, machine-mode start, supervisor-mode init
- **Memory**: Buddy allocator, Sv39 paging, lazy allocation, COW fork
- **Processes**: 64 process slots, round-robin scheduler, sleep/wakeup, process groups
- **Syscalls**: 107 syscalls including getenv, setenv, unsetenv, clearenv, getpagesize, sigaction, sigprocmask, sigpending, sigsuspend, sigreturn, killpg, setgroups, getgroups, initgroups, pathconf, fpathconf, sysconf, confstr, ttyname, ttyioctl, tcgetsid, tcflow, tcflush, mkfifo, pipe2, setpgid, getsid, setreuid, setregid, setresuid, setresgid, getresuid, getresgid, readv, writev, pread, pwrite, time, nanosleep, clock_gettime, mmap, munmap, mprotect, dup2, getppid, setuid, setgid, getpgid, isatty, etc.
- **Syscalls**: fork, exec, wait, exit, open, read, write, pipe, socket, etc.
- **Filesystem**: Log-structured with write-ahead logging, inode-based
- **Networking**: Ethernet, ARP, IPv4, UDP, DHCP, loopback, E1000 PCIe NIC (verified with QEMU user-mode NAT)
- **VirtIO**: Block device (disk)
- **Shell**: Pipes, redirections, background jobs, line editor with history

## Crate Names

- Kernel crate: `xv8`
- User library: `user`
- Kernel dependency in user: `kernel = { package = "xv8", path = "../kernel" }`

## QEMU Configuration

- Machine: virt
- CPU: max
- Memory: 256M
- SMP: 4 cores
- Network: QEMU user-mode NAT (`-netdev user,id=net0 -device e1000,netdev=net0`)
- E1000 NIC: Intel 82540EM (vendor=0x8086, device=0x100E), MMIO at 0x40000000

## Tests

17 tests across 4 categories, runnable individually or via `./test.sh`:

| Script | Tests | Depends on |
|--------|-------|------------|
| `test_core.sh` | fs, pipe, proc, fd, sbrk, cow, syscall (7) | user package only |
| `test_async.sh` | async, httpepoll, axum (3) | user package only |
| `test_net.sh` | net, neteth, netdns, tcpecho, nettools, http (6) | posix + net tools |
| `test_shell.sh` | shtest (1 = 93 shell tests) | posix + net tools |

```bash
./test.sh                    # All 17 tests (full QEMU run per category)
./test_core.sh               # Fast: kernel basics only (no posix/net build)
./test_async.sh              # Async runtime + tokio-compat (no posix/net build)
./test_net.sh                # Network: DHCP, DNS, TCP echo, httpd, tcpserver
./test_shell.sh              # Shell: 93 POSIX shell behavior tests
```

- Test filtering via `/test_args` file (comma-separated exact match prefixes)
- Core and async tests build only `--package user` (fast, no cross-compile)
- Net and shell tests use `./mkfs.sh` which includes all posix + net binaries
- Test runner forks + execs each test binary, collects pass/fail
- `shtest` runs xv8's shell against `shtest.sh` (93 TAP-format tests)

All 17 tests pass. Other xv8 user binaries: demo, primes, tcp_echo, udp, uptime, zombie.