# net — AI 輔助開發指南

## 專案概述

xv8 網路工具集，15 個網路工具 + 通訊協定函式庫 (libnet)。兩個獨立 crate。

| Project | Dir | 說明 |
|---------|-----|------|
| libnet | `libnet/` | 通訊協定函式庫 (DNS, ICMP, NTP, TFTP) |
| tools | `tools/` | 15 個網路工具二進位檔案 |

## 常用命令

```bash
# Host 編譯
cargo build --release

# RISC-V 交叉編譯
cargo build --release -p tools --no-default-features --features xv8 \
    -Zbuild-std=core,alloc --target riscv64gc-unknown-none-elf

# 測試
./test.sh
cd ../xv8 && ./test_net.sh    # 在 QEMU 中測試
```

## 關鍵限制

- **`no_std` + `no_main` wrapper** — RISC-V 目標需要 `rustc-wrapper.sh` 注入屬性 (透過 `.cargo/config.toml`)
- **`libc` → `xv8-libc-compat`** — `tools/Cargo.toml` 中的 `libc` 相依在 RISC-V 上解析為 `xv8-libc-compat`
- **Resolver** — `net/` 使用 resolver `"2"` (與 `posix/` 和 `xv8rust/` 不同)
- **QEMU 網路** — 需要 QEMU user-mode NAT (`-netdev user,id=net0`)

## 依賴鏈

```
tools (curl, ping, dns, ...)
  ├── libnet (dns, icmp, ntp, tftp 實作)
  ├── xv8-libc-compat (RISC-V syscall)
  └── xv8-user-std (io, net, time)
```

## 相關文件

- [README.md](README.md)
- [計劃版本記錄](_doc/)
- [xv8 網路核心](../xv8/kernel/src/net/)
- [xv8 AGENTS.md](../xv8/AGENTS.md)
