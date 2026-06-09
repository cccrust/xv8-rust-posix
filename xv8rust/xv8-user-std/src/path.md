# Path — 路徑處理

`path.rs` 實作 `std::path` 模組，提供跨平台的路徑操作抽象。

## PathBuf 與 Path

- **PathBuf**: 可變的路徑緩衝區（類似 `String`）
- **Path**: 不可變的路徑切片（類似 `str`）

xv8 使用 Unix 風格路徑（`/` 分隔），`Path` 實作直接委派給 `OsStr` 操作。

## 平台差異

xv8 上無相對路徑與絕對路徑的概念差異——所有路徑由核心的檔案系統層解析。xv8-user-std 的 `path` 模組遵循 Unix 路徑語義：`..` 表示父目錄，`.` 表示當前目錄，根目錄為 `/`。

## 相關文件

- [fs.md](./fs.md) — 檔案系統
- [ffi.md](./ffi.md) — 外部函式介面
