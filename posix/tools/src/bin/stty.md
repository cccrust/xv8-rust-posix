# Stty — 終端機設定

`stty`（set teletype / set tty）設定或顯示終端機參數。控制字元對映（如 `^C` 對 SIGINT、`^\` 對 SIGQUIT、`^Z` 對 SIGTSTP）、輸入模式（raw/cooked）、輸出處理（換行轉換）、波特率等。`stty -a` 顯示所有設定。此工具直接操作 termios 結構，修改驅動層的行為。

## 相關文件

- [tty.md](./tty.md) — 終端機名稱
- [tput.md](./tput.md) — 終端機能力設定
