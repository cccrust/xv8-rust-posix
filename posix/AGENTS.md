# posix - POSIX Utilities in Rust

A comprehensive collection of POSIX-compliant command-line utilities implemented in pure Rust.

## Project Structure

```
posix/
├── Cargo.toml          # Workspace manifest (members: libposix, tools)
├── Cargo.lock
├── libposix/          # Shared library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs     # Module root
│       ├── io.rs      # Read/Write trait definitions
│       ├── fmt.rs     # Formatting utilities
│       └── opt.rs     # Option parsing
├── tools/             # Command-line utilities
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs     # Library code
│   │   ├── bin/        # 100+ binary implementations
│   │   │   ├── echo.rs, cat.rs, ls.rs, cp.rs, mv.rs, rm.rs...
│   │   │   ├── sh.rs, grep.rs, sed.rs, awk.rs...
│   │   │   ├── vi.rs, ed.rs, more.rs, less.rs...
│   │   │   └── ... (many more)
│   │   └── tests/
│   │       └── basic.rs
└── _doc/              # Version history documentation
    ├── v0.1.md - v0.19.md
    ├── plan.md
    └── todo.md, todo2.md
```

## Crates

- **libposix** - Core library providing shared traits and utilities
  - `io::Read`, `io::Write` trait definitions
  - `fmt` - Formatting utilities
  - `opt` - Option parsing

- **tools** - Aggregated binary crate with 100+ utilities
  - Uses `libposix` for shared functionality
  - Depends on `libc` for system calls

## Test

```bash
# Run all tests
./test.sh

# Or run individually:
sh tools/tests/test_sh_basic.sh
sh tools/tests/test_tools_core.sh

# Both suites: PASS: 33/33 (shell) + PASS: 21/21 (core tools)
```

## Build

```bash
# Build entire workspace
cargo build --release

# Build specific crate
cargo build -p libposix
cargo build -p tools
```

## Features

- **Phase 1 - Basic I/O:** echo, true, false, yes, cat, wc, basename, dirname, sleep, kill, uname, whoami, id, hostname, printenv, env
- **Phase 2 - File Operations:** ls, cp, mv, rm, mkdir, rmdir, ln, touch, chmod, chown, chgrp
- **Phase 3 - Text Processing:** head, tail, sort, uniq, cut, tr, tee, od, cmp, diff
- **Phase 4 - Search & Filter:** grep, sed, xargs
- **Phase 5 - System Tools:** ps, du, df, stty, test, nice, nohup, date
- **Phase 6 - Shell:** sh (POSIX shell implementation)
- **Phase 7 - Advanced:** find, tar, patch, comm, fold, fmt, nl, expand, unexpand
- **Additional:** ed, awk, bc, m4, pax, vi, make, man, lp, mailx, su, cal, ed, and more

## Dependencies

- `libposix`: no external dependencies (can be `no_std`)
- `tools`: libc crate for system calls

## Documentation

Version history and development notes are in `_doc/`:
- v0.1.md through v0.19.md - Release notes
- plan.md - Development plan
- todo.md, todo2.md - Pending work