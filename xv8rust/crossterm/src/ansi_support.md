# ANSI 逸出碼偵測

偵測終端是否支援 ANSI 逸出碼序列。在 Unix 系統上檢查 `TERM` 環境變數
（如 `xterm`、`linux`、`screen` 等已知支援 ANSI 的終端類型），
並可透過 `NO_COLOR` 或 `FORCE_COLOR` 等環境變數覆蓋。Windows 上則查詢
Windows 10 的虛擬終端 API 支援。此模組是 crossterm 決定是否使用
ANSI 序列或 fallback API 的關鍵判斷點。
