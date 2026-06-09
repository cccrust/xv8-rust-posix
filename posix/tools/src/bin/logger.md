# Logger — 系統日誌工具

`logger` 將訊息寫入系統日誌（syslog）。Syslog（RFC 5424）是 Unix 系統的標準日誌框架，由 Eric Allman 在 1980 年代開發。`logger` 直接將訊息傳送給 syslogd 常駐程式。支援指定 facility（LOG_USER、LOG_DAEMON、LOG_AUTH）與優先級（LOG_INFO、LOG_ERR）。

## 相關文件

- [syslog.md](../../kernel/src/syslog.md) — 系統日誌
- [log.md](../../kernel/src/log.md) — 核心日誌
