# 事件過濾器 trait

定義 `EventFilter` trait，用於過濾事件串流中的特定事件型別。
支援的過濾條件包括：按事件類型（Keyboard、Mouse、Resize）、
按鍵碼（KeyCode）、修飾鍵（Modifier）組合等。
`EventStream` 可使用 `filtered()` 方法套用過濾器，只傳遞符合條件的事件，
減少應用層級的事件處理複雜度。
