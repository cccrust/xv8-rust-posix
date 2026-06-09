# File — 偵測檔案類型

`file` 命令通過檢查檔案的魔術數字（magic number/ magic byte）與其他特徵來推測檔案類型。Unix 從 V7 開始使用 `/etc/magic` 檔案定義魔術數字與檔案類型的對應表。`file` 在開頭讀取前幾個位元組，比對已知檔案格式的簽名（如 `%PDF`、`\x89PNG`、`ELF`），輸出人類可讀的檔案類型描述。

## 相關文件

- [ls.md](./ls.md) — 列出檔案
- [stat.md](./stat.md) — 檔案狀態查詢
