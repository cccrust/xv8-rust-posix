# Windows 事件來源 stub

Windows 平台上事件來源的 stub 實作。實際的 Windows 事件來源
應使用 `ReadConsoleInput` API 從控制台輸入緩衝區讀取 INPUT_RECORD，
並轉換為 crossterm 的 `Event` 型別。此 stub 保留介面但不提供
實際的 Windows Console 事件處理功能。
