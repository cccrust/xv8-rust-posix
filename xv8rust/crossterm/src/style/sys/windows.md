# Windows 樣式 stub

Windows 平台的樣式設定 stub。在舊版 Windows Console API 中，
樣式需透過 `SetConsoleTextAttribute` 設定，顏色僅支援 16 色。
Windows 10 以上可透過虛擬終端序列（VT processing）使用完整 ANSI SGR，
但此移植版本作為 stub，保持介面相容但不提供實際的 Win32 API 呼叫。
