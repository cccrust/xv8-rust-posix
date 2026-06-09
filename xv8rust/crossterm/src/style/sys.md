# 樣式系統模組分派器

根據平台分派樣式設定操作。在 Unix 上，樣式操作完全透過 ANSI SGR 逸出碼
輸出到標準錯誤或輸出串流；Windows 上則需要額外處理（如偵測是否為舊版
Console API）。此分派器封裝了 `SetForgroundColor`、`SetBackgroundColor`、
`SetAttribute` 等指令的平台特定行為，統一上層 API 介面。
