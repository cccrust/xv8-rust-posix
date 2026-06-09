# Gettext — 國際化工具

`gettext` 是 GNU 國際化（i18n）系統的核心工具。程式使用 `gettext("Hello")` 包裹需翻譯的字串，`gettext` 工具從 `.po`（Portable Object）檔案中查找對應的翻譯。`gettext` 命令本身用於測試訊息查找，也可從 shell 腳本中使用。

## 相關文件

- [msgfmt.md](./msgfmt.md) — .po → .mo 編譯
- [ngettext.md](./ngettext.md) — 複數形式處理
- [gencat.md](./gencat.md) — 類別目錄產生器
- [locale.md](./locale.md) — 語言環境
