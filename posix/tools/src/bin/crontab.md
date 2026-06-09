# Crontab — 定期任務排程

`crontab` 管理 cron 定期任務表格。cron 是 Unix 的作業排程器，源自 V7 Unix，由 Ken Thompson 實作。每行 crontab 條目指定分鐘、小時、日、月、星期與要執行的命令。任務由 cron 常駐程式在指定時間執行。`crontab -e` 編輯、`-l` 列出、`-r` 移除。

## 相關文件

- [at.md](./at.md) — 一次性任務排程
- [batch.md](./batch.md) — 低負載排程
