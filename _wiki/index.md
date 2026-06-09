# xv8-rust-posix 技術 wiki

本 wiki 收錄 xv8-rust-posix 專案的核心概念、架構設計與專有名詞解釋。

## 專案架構

xv8-rust-posix 是一個多專案的 Rust 工作區，包含：
- **xv8 OS**：RISC-V Unix-like 作業系統
- **posix/**：124 POSIX 工具 + shell
- **xv8rust/**：std 覆寫層與 async runtime
- **net/**：網路工具（ping, dns, tcp, echo, ntp, whois, curl, ssh）

## 目錄索引

### 核心作業系統概念

| 詞項 | 說明 |
|------|------|
| [[RISC-V]] | xv8 目標的 CPU 架構 |
| [[Syscall]] | 使用者空間與核心間的系統呼叫介面 |
| [[Process]] | 程序管理、行程排程與生命週期 |
| [[Virtual-Memory]] | Sv39 分頁虛擬記憶體系統 |
| [[File-System]] | 日誌結構檔案系統（WAL） |
| [[Network-Stack]] | Ethernet、ARP、IPv4、UDP、DHCP 網路堆疊 |
| [[Trap]] | RISC-V 上的 Trap 處理與中斷 |
| [[Device-Drivers]] | UART、VirtIO、E1000 驅動程式架構 |
| [[Scheduler]] | 行程排程器與上下文切換 |

### 容器技術

| 詞項 | 說明 |
|------|------|
| [[Container]] | 容器隔離技術總覽 |
| [[Namespace]] | 7 種 Namespace 類型與 NsProxy 模型 |
| [[cgroup]] | 資源控制群組 (CPU/Memory/PIDs) |
| [[OverlayFS]] | 疊合檔案系統與容器分層儲存 |
| [[seccomp]] | 安全計算模式與 BPF 過濾器 |
| [[Capability]] | 能力模型與五集合權限管理 |

### POSIX 工具相關

| 詞項 | 說明 |
|------|------|
| [[Shell]] | POSIX shell（sh 與 bash）實作 |
| [[libc-compat]] | RISC-V 上的 libc 相容層 |
| [[xv8-std]] | std 覆寫層機制 |
| [[Cross-Compilation]] | 從 x86 主機交叉編譯 riscv64 二進位檔 |
| [[Rust-no_std]] | 無標準庫的 Rust 程式設計 |

### 測試與執行環境

| 詞項 | 說明 |
|------|------|
| [[QEMU]] | 用於測試 xv8 的 QEMU 模擬器 |

## 更新日誌

### 2026-06-09
- 新增容器技術系列：Container、Namespace、cgroup、OverlayFS、seccomp、Capability
- 更新 wiki 索引

### 2026-06-03
- 建立 wiki 結構，收錄 16 個核心 OS 概念與工具文件
