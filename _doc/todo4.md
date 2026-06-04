# xv8rust async runtime 開發規劃

## ✅ v1.1（已完成）

- `xv8-async`：單執行緒 executor、`block_on`、`spawn`、`JoinHandle`、`Sleep`、`YieldNow`、timer reactor
- `xv8-axum-smoke`：host 端 axum HTTP server → 打 localhost request 驗證路徑

---

## ✅ v2.1（已完成）

**xv8-async riscv64 編譯 + IO reactor**

- `xv8-async` 可在 `riscv64gc-unknown-none-elf` 下編譯通過
- 新增 `AsyncTcpListener` / `AsyncTcpStream` / `ReadFuture` / `WriteFuture` async wrappers
- 目前 blocking IO 直接回傳 `Poll::Ready`（kernel 無 epoll/select）
- `xv8-axum-smoke` 暫時從 workspace 移除（v2.3 重新加入）

---

## ✅ v2.2（已完成）

**QEMU async smoke test**

- `posix/tools/src/bin/async_echo.rs`：xv8-async TCP echo server，監聽 port 27001
- `xv8/user/testbin/async.rs`：QEMU 整合測試（fork + exec async_echo，TCP send/recv echo 驗證）
- 納入 testrunner 測試清單（`/_async` 為第 12 項）
- 透過 mkfs.sh 自動嵌入 async_echo 至 fs.img

---

## ✅ v2.3（已完成）

**xv8-axum-smoke riscv64 — 最小 HTTP server**

- **依賴分析**：axum 完整移植不可行（tokio→mio→OS event, hyper, http 型別, 本地 fork 路徑不存在）
- **替代方案**：自製最小 HTTP router，使用 xv8-async 實作 TCP listener + 手動 HTTP 回應
- `posix/tools/src/bin/axum_smoke.rs`：xv8-async HTTP server（GET / → 200 "ok"）監聽 port 27003
- `xv8/user/testbin/http.rs`：QEMU 整合測試（fork + exec axum_smoke，HTTP GET 驗證 200 OK + "ok"）
- 納入 testrunner 測試清單（`/_http` 為第 13 項）

---

## 長期方向

| 項目 | 優先級 | 依賴 |
|------|--------|------|
| xv8-async IO reactor (poll/select syscall) | 中 | v2.2 |
| tokio API 相容面補齊（time::Sleep, net::TcpStream） | 低 | v2.1 |
| xv8-async → posix 工具 async 化 | 低 | 穩定後 |
