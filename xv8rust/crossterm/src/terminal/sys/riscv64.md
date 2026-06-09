# RISC-V 終端 stub

xv8 核心上的終端 stub 實作。由於 xv8 的 UART 序列埠不支援 termios 或
ioctl 操作，所有終端功能（原始模式、尺寸查詢、畫面清除）皆為空操作。
原始模式 stub 回傳成功但實際上不改變終端設定；尺寸查詢回傳固定值
（如 80×24）。此 stub 讓依賴 crossterm 的程式（如 vi/vim）
可在 xv8 上編譯通過，儘管互動體驗受限。
