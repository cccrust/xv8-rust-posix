# Lib — 網路模組

`lib.rs` 是 `xv8-net` crate 的根模組，提供與 `std::net` 相容的網路功能。

## 設計目標

xv8-net 提供可移植的網路抽象，在 xv8 環境中底層使用 xv8-libc 的系統呼叫，在 host 環境中則使用標準 `std::net`。此 crate 透過條件編譯根據目標平台選擇實作後端。

## 重新匯出

`lib.rs` 公開 `net.rs` 中定義的所有類型，並根據平台特性補齊必要的擴充功能（如 `ToSocketAddrs` trait）。

## 相關文件

- [net.md](./net.md) — 核心網路類型
- [net.md](../../xv8-user-std/src/net.md) — xv8 std 網路模組
