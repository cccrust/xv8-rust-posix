# Windows 事件喚醒器 stub

Windows 事件喚醒器的 stub 實作。實際實作應使用 Windows Event 物件
（`CreateEvent`/`SetEvent`）來中斷 `WaitForMultipleObjects` 的等待狀態。
此喚醒器用於在 Windows 上中斷事件輪詢，支援應用程式的優雅關閉
與外部事件注入。
