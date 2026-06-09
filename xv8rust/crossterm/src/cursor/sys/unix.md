# Unix 游標實作

透過 ANSI 逸出碼與終端裝置互動。使用 `\x1b[6n`（DSR——裝置狀態報告）
查詢游標位置，從 stdin 解析 `\x1b[row;colR` 格式的回應。
儲存/恢復位置使用 DECSC/DECRC（`\x1b7`/`\x1b8`），
顯示/隱藏使用 DECTCEM（`\x1b[?25h`/`\x1b[?25l`）。
這些逸出序列在 xterm、gnome-terminal 等主流終端模擬器上都支援。
