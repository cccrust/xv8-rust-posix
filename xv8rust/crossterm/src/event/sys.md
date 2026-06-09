# 事件系統模組分派器

根據平台分派事件的底層 I/O 操作。Unix 平台使用 `unix` 模組讀取 stdin
檔案描述子並解析 ANSI/CSI 逸出碼序列；Windows 平台使用 `windows` 模組
中的 Console API 讀取輸入記錄。此分派器負責將原始位元組序列轉換為
高層的 `Event` 型別（`Event::Key`、`Event::Mouse`、`Event::Resize`）。
