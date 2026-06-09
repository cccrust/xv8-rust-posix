# Lib — Tokio 相容層

`lib.rs` 是 xv8-tokio-compat crate 的根模組，提供與 tokio 生態系相容的非同步執行環境。

## 相容性策略

此 crate 不完整實作 tokio 的全部 API，而是專注於讓 xv8 上的程式可以使用 tokio 風格的 async 程式設計。它實作以下元件的介面相容版本：

- **tokio::io**: AsyncRead/AsyncWrite trait
- **tokio::runtime**: 簡化的執行期
- **tokio::sync**: 同步原語
- **tokio::time**: 時間操作

## 使用場景

讓 xv8rust 中的 HTTP router、network tools 等元件（原本依賴 tokio）可在 xv8 上運作，無需大幅修改程式碼。

## 相關文件

- [runtime.md](./runtime.md) — 執行期元件
- [io.md](./io.md) — I/O 相容層
- [sync.md](./sync.md) — 同步相容層
- [time.md](./time.md) — 時間相容層
