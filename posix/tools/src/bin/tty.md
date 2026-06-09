# Tty — 顯示終端機名稱

`tty`（teletype）顯示連接到標準輸出的終端機裝置名稱（如 `/dev/tty1`、`/dev/pts/3`）。陳述返回碼：若 stdin 是終端機則傳回 0（成功），否則傳回 1。用於腳本中檢測是否為互動式執行環境。

## 相關文件

- [stty.md](./stty.md) — 終端機設定
- [tput.md](./tput.md) — 終端機能力查詢
- [mesg.md](./mesg.md) — 訊息控制
