# RISC-V 游標 stub

xv8 核心上的 stub 實作。由於 xv8 的終端是透過 UART 序列埠，
不支援 ANSI 游標控制序列，所有游標操作皆為空操作（no-op）。
此 stub 確保 crossterm 可在 xv8 RISC-V 目標上編譯通過，
但在序列埠模式下不會實際移動游標或改變其可見性。
