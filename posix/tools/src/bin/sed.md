# Sed — 串流編輯器

`sed`（stream editor）是非互動式的串流編輯器，由 Lee E. McMahon 於 1973–74 年在 Bell Labs 開發。`sed` 逐行讀取輸入，應用編輯規則（script），輸出結果。基本操作包括 `s/old/new/g`（取代）、`d`（刪除）、`p`（列印）、`a/i/c`（追加/插入/變更行）。Sed 使用正則表達式進行模式匹配。著名的 `sed 's/foo/bar/g'` 是所有 Unix 使用者的起點。

## 相關文件

- [awk.md](./awk.md) — 模式掃描語言
- [grep.md](./grep.md) — 模式匹配
- [ed.md](./ed.md) — 行編輯器
