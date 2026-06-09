# Gencat — 訊息類別目錄產生器

`gencat`（generate catalog）將訊息來源檔案轉換為訊息類別目錄（message catalog）二進位格式。訊息類別目錄是國際化（i18n）的基礎，將程式中的文字訊息與語言環境分離。`gencat` 產生 `.cat` 檔案，應用程式在執行期透過 `catgets` 讀取對應語言的訊息。

## 相關文件

- [msgfmt.md](./msgfmt.md) — GNU gettext 訊息格式工具
- [locale.md](./locale.md) — 語言環境設定
- [gettext.md](./gettext.md) — 國際化工具
