# 事件來源分派器

根據平台與功能特徵分派事件來源的實作。支援三種來源：Unix 上的 mio 非同步後端、
TTY 檔案描述子輪詢、以及 Windows Console API。
事件來源負責初始化的設定（如將終端設為滑鼠事件模式）、
底層 I/O 的輪詢（poll）、以及事件發生時的通知喚醒（wake）。
上層 `EventStream` 與 `InternalEventBus` 依賴此分派器取得平台對應的來源。
