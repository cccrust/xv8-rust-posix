# Nohup — 忽略掛斷訊號執行

`nohup`（no hang up）執行命令並忽略 SIGHUP（掛斷）訊號。當使用者登出 shell 時，shell 向它所屬的行程傳送 SIGHUP，導致後者終止。`nohup` 包裝命令使其不受登出影響。輸出自動重新導向到 `nohup.out`。

## 相關文件

- [bg.md](./bg.md) — 背景執行
- [disown.md](./disown.md) — Shell 工作排除
- [signal.md](../../kernel/src/signal.md) — 訊號處理
