# Overlay — OverlayFS 測試

`overlay` 測試驗證 xv8 的 OverlayFS 聯合檔案系統功能。OverlayFS 將兩個目錄（upper 與 lower）疊合成一個合併的視圖，對其的寫入僅影響 upper 層。這是 Docker 等容器引擎容器映像的底層技術——多個容器共用底層唯讀映像層，每個容器擁有自己的可寫上層。

## 相關文件

- [overlay.md](../../kernel/src/overlay.md) — 核心 overlay 實作
- [dock8.md](../bin/dock8.md) — 容器 CLI
- [container.md](./container.md) — 容器測試
