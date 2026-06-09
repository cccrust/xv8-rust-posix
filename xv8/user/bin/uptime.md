# Uptime — 系統運行時間顯示

`uptime` 顯示系統從啟動到目前的運行時間，以及其他系統負載資訊。

## 系統時間

`uptime` 讀取核心維護的 tick 計數器（timer interrupt 觸發次數），將其轉換為人類可讀的時間格式：

```
系統已運行： 0 days, 1:23:45
```

在 Unix 系統中，`uptime` 也顯示登入使用者數量與 CPU 平均負載（1、5、15 分鐘），但 xv8 精簡版的 `uptime` 主要展示系統時鐘與時間系統呼叫。

## 系統呼叫

`uptime` 透過 `uptime` 系統呼叫或讀取 `/proc/uptime` 取得啟動以來的秒數。這依賴 xv8 的時鐘中斷處理器正確維護全域時間計數。

## 相關文件

- [time.md](../../../xv8rust/xv8-user-std/src/time.md) — 時間模組
- [timerfd.md](../testbin/timerfd.md) — Timerfd 測試
