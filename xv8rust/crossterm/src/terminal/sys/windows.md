# Windows 終端 stub

Windows 終端操作的 stub 實作。實際的 Windows 實作應使用
`GetStdHandle`、`SetConsoleMode`、`GetConsoleScreenBufferInfo` 等
Win32 API 來操作控制台。此 stub 保留介面相容性，
但所有操作皆為空操作，不會改變 Windows 控制台的設定。
