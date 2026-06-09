# 事件讀取

提供 `read()` 與 `read_line()` 等功能，從事件來源同步讀取下一個事件。
支援 Blocking 模式（無事件時阻塞等待）以及設定逾時的讀取。
此模組包裝了底層的 `InternalEventBus`，將內部位元組串流
轉換為高層的 `Event` 型別。`read` 是事件消費的主要入口，
適用於不需要 `EventStream` 非同步架構的簡單場景。
