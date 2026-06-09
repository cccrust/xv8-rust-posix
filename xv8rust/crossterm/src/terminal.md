# 終端模組

提供終端層級的操作：查詢終端尺寸（行數與欄數）、啟用/停用原始模式
（raw mode，關閉行緩衝與字元處理）、啟用/停用替代畫面緩衝區、
清除畫面、捲動等。原始模式是 TUI 程式的基礎——關閉終端的 ICANON、
ECHO、ISIG 等旗標，讓程式可以直接讀取每個按鍵輸入。
此模組在 Unix 上透過 `tcgetattr`/`tcsetattr` 操作 termios 結構體，
在 Windows 上透過 `SetConsoleMode` API 實作。
