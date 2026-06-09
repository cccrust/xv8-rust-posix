# 事件串流

實現 `futures::Stream` trait 的非同步事件串流。`EventStream` 包裝了
事件來源，實作 `poll_next()` 方法以支援 .await 語法。
支援透過 `filter()`、`map()` 等組合子進行事件轉換，
以及 `filtered()` 方法套用 `EventFilter`。
此模組是 crossterm 非同步事件處理的核心，讓終端事件能與 tokio 等
async runtime 整合。
