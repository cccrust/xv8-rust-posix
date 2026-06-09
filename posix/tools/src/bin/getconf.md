# Getconf — 系統設定值查詢

`getconf`（get configuration）查詢系統限制與設定值（如 PATH_MAX、PAGE_SIZE、LONG_BIT）。這些值定義在 POSIX 的 `<limits.h>` 與 `<unistd.h>` 中。`getconf` 提供一個標準化的查詢介面，讓 shell 腳本與程式可取得系統級常數與執行期可變的值。

## 相關文件

- [uname.md](./uname.md) — 系統資訊
- [locale.md](./locale.md) — 語言環境
