# Patch — 套用補丁

`patch` 將 diff（patch 檔案）套用到原始檔案上，用於檔案更新與版本控制。Patch 檔案由 `diff` 工具產生，包含增刪行的上下文。`patch` 搜尋匹配的上下文以定位修改位置（即使行號有偏移），這稱為模糊匹配（fuzzy matching）。Larry Wall 於 1985 年編寫最初的 `patch`。

## 相關文件

- [diff.md](./diff.md) — 檔案差異比較
