# Windows 輪詢 stub

Windows 事件輪詢的 stub 實作。實際實作應使用 `WaitForMultipleObjects`
或 `GetNumberOfConsoleInputEvents` 來檢查是否有待處理的輸入事件，
而不需阻塞呼叫 `ReadConsoleInput`。此 stub 保留介面供未來擴展。
