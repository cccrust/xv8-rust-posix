# Join — 關聯檔案合併

`join` 命令將兩個檔案中具有相同鍵（key）的行合併為單行，類似 SQL 的 JOIN 操作。輸入檔案需以鍵排序。`join` 支援內部連接（inner join）、左連接（left outer join）等。鍵欄位依 1-based 索引指定。這是 Unix 文字處理工具箱的老牌成員，與 `sort` 密切配合。

## 相關文件

- [sort.md](./sort.md) — 排序
- [comm.md](./comm.md) — 比較排序檔案
- [uniq.md](./uniq.md) — 移除重複行
