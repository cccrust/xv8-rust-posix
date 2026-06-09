# Runtime — 執行期初始化

`runtime.rs` 實作 xv8 使用者程式的執行期初始化（runtime initialization）。在 Rust 的 `fn main()` 執行之前，需要完成一系列初始化工作。

## 初始化序列

Rust 在 `_start` 入口點由 crt0（C runtime zero）設定堆疊後，依序執行：

1. **argc/argv 解析**: 從核心設定的使用者堆疊中提取引數
2. **環境變數初始化**: 建立環境變數字串表
3. **heap 初始化**: 設定 `__rust_alloc` 的初始堆積範圍
4. **thread local storage**: 初始化 TLS 區域
5. **main 呼叫**: 跳轉到使用者的 `fn main()`

## xv8 的適應

xv8-user-std 的 runtime 初始化精簡了標準 runtime 的許多步驟，省略了並未使用的功能（如 GC、tracing）。堆積初始化直接呼叫 `sbrk` 系統呼叫獲得初始記憶體區塊。

## 相關文件

- [lib.md](./lib.md) — 總覽
- [process.md](./process.md) — 行程管理
