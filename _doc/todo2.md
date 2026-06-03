# xv8-rust-posix 開發規劃 v2.0+

從 v2.0 開始，聚焦網路功能完備（含 http、ssl 等工具）。

---

## v2.0 — users/who 強化 + 小型選用工具

| 項目 | 子專案 | 說明 |
|------|--------|------|
| `users` 強化 | posix | 支援 `-n`、從 utmp 讀取 |
| `who` 強化 | posix | 支援 `-a`、`-b`、`-d`、`-H`、`-l`、`-p`、`-q`、`-r`、`-s`、`-t`、`-T`、`-u` |
| `last` 實作 | posix | 顯示最近登入記錄 |
| `logname` 強化 | posix | 支援 `-s`（syslog 模式） |
| `look` 實作 | posix | 字典查詢工具 |
| `rev` 實作 | posix | 反轉每一行字元 |
| `col` 實作 | posix | 過濾反向換行 |

## v2.1 — xv8 核心網路強化

| 項目 | 子專案 | 說明 |
|------|--------|------|
| TCP 協定支援 | xv8 | 核心網路堆疊加入 TCP |
| HTTP 伺服器/客戶端 | xv8 | 核心層 HTTP 支援 |
| TLS/SSL 支援 | xv8/net | mbedTLS 移植或簡易加密層 |
| 動態行程表 | xv8 | 不再固定 64 個行程槽 |
| SysV IPC | xv8 | 共享記憶體、號誌、訊息佇列 |

## v2.2 — 網路工具整合

| 項目 | 子專案 | 說明 |
|------|--------|------|
| curl/wget 實作 | net/xv8 | HTTP(S) 下載工具 |
| http_server 強化 | net/xv8 | 完整 HTTP 伺服器 |
| ssh_client/sshd 實作 | net/xv8 | SSH 客戶端與伺服器 |
| DNS over TLS | net/xv8 | 加密 DNS 查詢 |
| QEMU 內網路測試 | xv8 | ping/dns/ntp/http 在 QEMU 內執行 |
| 防火牆工具 | xv8 | 簡易封包過濾 |

## 版本對照

| 版本 | 主要內容 | 優先度 |
|------|---------|--------|
| v2.0 | users/who + 小型選用工具 | 高 |
| v2.1 | xv8 核心網路強化（TCP、HTTP、TLS） | 高 |
| v2.2 | 網路工具整合（curl、ssh、https） | 高 |