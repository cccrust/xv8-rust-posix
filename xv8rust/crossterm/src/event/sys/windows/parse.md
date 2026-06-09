# Windows Console API 解析 stub

Windows 控制台 API 輸入記錄的解析 stub。實際實作應將
`ReadConsoleInput` 回傳的 `INPUT_RECORD` 結構（包含 `KEY_EVENT_RECORD`、
`MOUSE_EVENT_RECORD`、`WINDOW_BUFFER_SIZE_RECORD`）轉換為 `Event`。
此 stub 保留介面，但因主要目標為 Unix/RISC-V，未實作完整功能。
