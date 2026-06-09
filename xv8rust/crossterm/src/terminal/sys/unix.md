# Unix 終端實作

透過 termios 與 ioctl 系統呼叫操作終端。核心功能包括：
**原始模式**——使用 `tcsetattr` 清除 ICANON（行緩衝）、ECHO（字元回顯）、
ISIG（訊號產生）、IEXTEN、OPOST 等旗標，並設定 VMIN=1、VTIME=0 實現即時讀取；
**尺寸查詢**——透過 `TIOCGWINSZ` ioctl 取得 `winsize` 結構的 ws_row 與 ws_col；
**畫面清除**——輸出 ANSI `\x1b[2J`（清除整個畫面）與 `\x1b[3J`（清除捲動緩衝區）。
