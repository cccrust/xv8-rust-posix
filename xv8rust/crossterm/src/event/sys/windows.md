# Windows 事件系統 stub

Windows 事件系統的 stub 實作。實際實作應使用 `ReadConsoleInput` API
讀取 `INPUT_RECORD` 陣列，並將 `KEY_EVENT`、`MOUSE_EVENT`、
`WINDOW_BUFFER_SIZE_EVENT` 等記錄轉換為 crossterm 的 `Event` 型別。
此 stub 保留介面簽名，但不提供實際的 Windows 事件讀取功能。
