# Msgfmt — 訊息格式編譯器

`msgfmt`（message format）將 GNU gettext 的 `.po`（Portable Object）文字檔案編譯為二進位 `.mo`（Machine Object）格式。`.mo` 檔案可在執行期被 `gettext()` 函式快速載入查閱。此編譯過程包括訊息字串的 hash 表建立與字串資料壓縮。

## 相關文件

- [gettext.md](./gettext.md) — 國際化工具
- [ngettext.md](./ngettext.md) — 複數形式處理
- [gencat.md](./gencat.md) — 訊息類別目錄
