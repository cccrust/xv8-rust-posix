# 剪貼簿操作

提供讀取與寫入系統剪貼簿的功能。在 Unix 系統上透過外部指令
（如 `xclip` 或 `pbpaste`/`pbcopy`）實作；Windows 上使用 Win32 API。
此模組封裝了平台特定的剪貼簿存取細節，提供統一的 `get`/`set` 介面。
在 xv8 RISC-V 平台上，由於缺乏圖形化剪貼簿系統，此模組為 stub 實作。
