# Ar — 靜態函式庫歸檔工具

`ar`（archiver）建立、修改與提取靜態函式庫（歸檔檔案）。歸檔檔案將多個目標檔案（`.o`）合併為單一 `.a` 檔案，連結器（linker）可從中提取所需符號。`ar` 使用特定的歸檔格式（Unix ar format），包含每個成員的標頭（檔名、時間戳、UID、GID、權限、大小）。

## 相關文件

- [nm.md](./nm.md) — 符號列表
- [strings.md](./strings.md) — 字串提取
