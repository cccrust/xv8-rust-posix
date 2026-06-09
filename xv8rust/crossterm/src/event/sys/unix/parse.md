# Unix 事件解析

將原始 ANSI/CSI 逸出序列解析為結構化的 `Event` 型別。支援的序列包括：
**CSI 序列**（以 `\x1b[` 開頭）——方向鍵、Home/End、Insert/Delete、
F1–F12 功能鍵、游標位置回報（CPR）；
**SS3 序列**（以 `\x1bO` 開頭）——部分功能鍵的替代編碼；
**滑鼠事件**（SGR 編碼格式 `\x1b[<row;col;btnM/m`）；
**終端尺寸變更**（從 SIGWINCH 或 `\x1b[8;row;col;t` 序列取得）。
解析器使用有限狀態機（FSM）逐步消費位元組，確保部分序列的累積與超時處理。
