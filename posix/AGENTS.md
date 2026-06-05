# posix - POSIX Utilities in Rust

A comprehensive collection of POSIX-compliant command-line utilities implemented in pure Rust.

## Project Structure

```
posix/
├── Cargo.toml          # Workspace manifest (members: ["tools"])
├── Cargo.lock
├── tools/             # Single crate with all 124 utilities
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs     # Shared library code
│   │   ├── bin/        # 124 binary implementations
│   │   │   ├── echo.rs, cat.rs, ls.rs, cp.rs, mv.rs, rm.rs...
│   │   │   ├── sh.rs, grep.rs, sed.rs, awk.rs...
│   │   │   ├── vi.rs, ed.rs, more.rs, less.rs...
│   │   │   └── ... (many more)
│   │   └── tests/
│   │       └── basic.rs
├── .cargo/config.toml  # riscv64 cross-compile config (stashed for host builds)
└── _doc/              # Version history documentation

## Test

```bash
# Run all tests
./test.sh

# Or run individually:
sh tools/tests/test_sh_basic.sh      # 33 shell tests
sh tools/tests/test_tools_core.sh    # 21 core tool tests
sh tools/tests/test_v2.0.sh          # v2.0 tools (rev, col, look, who, users, last)

# Rust integration test
cargo test --release
```

## Build

```bash
# Build entire workspace
cargo build --release

# Build the tools crate (only member)
cargo build -p tools --release
```

## Features

- **Phase 1 - Basic I/O:** echo, true, false, yes, cat, wc, basename, dirname, sleep, kill, uname, whoami, id, hostname, printenv, env
- **Phase 2 - File Operations:** ls, cp, mv, rm, mkdir, rmdir, ln, touch, chmod, chown, chgrp
- **Phase 3 - Text Processing:** head, tail, sort, uniq, cut, tr, tee, od, cmp, diff
- **Phase 4 - Search & Filter:** grep, sed, xargs
- **Phase 5 - System Tools:** ps, du, df, stty, test, nice, nohup, date
- **Phase 6 - Shell:** sh (POSIX shell implementation)
  - **Builtins:** `:` `cd` `echo` `eval` `exec` `exit` `export` `readonly` `read` `return` `set` `shift` `test` `[` `trap` `type` `unset` `wait` `command` `.` `source` `alias` `unalias` `break` `continue`
  - **Features:** pipes, redirects (`>` `>>` `<` `<<` `<<<` `<>` `>&` `>&-` `2>` `2>&1`), variable expansion (`$var` `${var:-}` `${var:=}` `${var:+}` `${var:?}` `${#var}` `${var%}` `${var%%}` `${var#}` `${var##}`), arithmetic `$((...))`, command substitution `$(...)`, glob `*?[`, `if`/`then`/`elif`/`else`/`fi`, `for`/`in`/`do`/`done`, `while`/`until`, `case`/`|`/`esac`, functions `name() { }`, source/PATH search, `set -e`/`-u`/`-C`/`-x`/`-v`/`-n`/`-f`/`-m`, readline/history (basic), heredoc, alias, subshell `( )`, brace group `{ }`
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