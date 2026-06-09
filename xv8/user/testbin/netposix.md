# Netposix — POSIX 網路 API 測試

`netposix` 測試驗證 POSIX socket API 在 xv8 上的完整實作，包括 `socket()`、`bind()`、`listen()`、`accept()`、`connect()`、`send()`/`recv()`、`close()`、`getsockname()` 與 `getpeername()`。此測試確認 xv8 的網路系統呼叫符合 POSIX.1-2001 標準，可與標準網路程式相容。

## 相關文件

- [net.md](./net.md) — 網路基礎測試
- [sysnet.md](../../kernel/src/sysnet.md) — 網路系統呼叫
- [abi.md](../../kernel/src/abi.md) — ABI 定義
