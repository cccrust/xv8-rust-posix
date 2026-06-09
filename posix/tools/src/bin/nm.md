# Nm — 符號表列表

`nm`（name list / symbol table）顯示目標檔案（`.o`、`.a`、ELF 可執行檔）中的符號表。符號分為多種類型：T（文字段、函式）、D（已初始化資料）、B（未初始化資料 BSS）、U（未定義、需連結器解析）。`nm` 用於偵錯連結錯誤、檢查函式存在與確認符號可見性。

## 相關文件

- [ar.md](./ar.md) — 靜態函式庫
- [strings.md](./strings.md) — 字串提取
- [strip.md](./strip.md) — 移除符號
