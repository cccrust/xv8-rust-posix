# RNG — 隨機數產生器

## 概述

`rng` 模組提供核心層級的隨機數產生能力。在作業系統中，隨機數用於 ASLR（位址空間佈局隨機化）、TCP 初始序號、cookie/token 產生等安全性關鍵場景。

## RISC-V 隨機 CSR

RISC-V 架構定義了 `mseccfg` 與 `mventry` 等 CSR，但真正的硬體隨機數介面來自於 **Zkr（Entropy Source）擴展**。Zkr 提供 `getentropy` 指令或 `seed` CSR，核心可藉此讀取硬體熵源。

在 QEMU 模擬環境中，這些 CSR 回傳的隨機值由 host 提供。QEMU 支援將 host 的 `/dev/urandom` 映射為 guest 的熵源。

## 熵池（Entropy Pool）

xv8 的 `rng` 模組維護一個內部熵池，從多個來源收集不確定性：

1. **硬體熵源**：RISC-V `rdseed` / `getentropy`（若硬體支援）
2. **中斷時間戳**：裝置中斷到達時間的微秒級抖動
3. **磁碟轉速抖動**：virtio 磁碟操作的完成時間變化
4. **網路封包到達時間**：網路中斷的間隔變化

## 使用場景

- **ASLR**：行程載入時隨機化堆疊、堆積、mmap 的基底位址
- **TCP ISN**：TCP 連線初始序號必須不可預測（RFC 6528）
- **核心 cookie**：檔描述序號、PID 產生等的亂數元件

## 相關文件

- [riscv.md](./riscv.md) — RISC-V 架構相關說明
- [tcp.md](../net/tcp.md) — TCP 協定實作
