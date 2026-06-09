# Lib — POSIX 工具共用函式庫

`lib.rs` 是 POSIX 工具集（tools crate）的共用函式庫，提供所有 144 個 POSIX 工具命令所需的輔助功能。

## 功能範圍

- **錯誤處理**: 統一的 errno 處理與錯誤訊息輸出
- **I/O 輔助**: 緩衝讀寫、行處理、檔案複製的通用實作
- **選項解析**: POSIX 風格的命令列選項處理（getopt-like）
- **系統呼叫包裝**: 對 xv8-libc-compat 系統呼叫的進一步封裝

## 設計目標

各工具命令（如 cp、mv、rm）共用此函式庫避免重複程式碼。函式庫保持輕量，僅包含各工具實際需要的輔助函式。

## 相關文件

- [sh.md](./bin/sh.md) — Shell 實作
- [raw.md](../../../xv8rust/xv8-libc/src/raw.md) — 系統呼叫包裝
- [lib.md](../../../xv8rust/xv8-libc-compat/src/lib.md) — libc 相容層
