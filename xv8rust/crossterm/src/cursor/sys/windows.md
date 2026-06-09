# Windows 游標 stub

Windows 上游標操作的實際實作依賴 Win32 Console API 而非 ANSI 逸出碼。
此模組提供 `CONSOLE_CURSOR_INFO` 與 `SetConsoleCursorPosition` 等 API 的封裝。
然而在此移植中，由於主要目標為 Unix 與 RISC-V，Windows 實作為 stub，
保留介面但不提供完整功能。
