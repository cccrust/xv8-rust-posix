# Pivot Root — 根檔案系統切換測試

`pivot_root` 測試驗證 `pivot_root` 系統呼叫，該呼叫將當前行程的根檔案系統切換到新的目錄，同時將舊的根檔案系統移到另一個目錄。此機制是容器啟動的關鍵步驟：容器引擎在建立新的 mount namespace 後使用 pivot_root 切換到容器映像的檔案系統，確保容器行程無法存取宿主檔案系統。

## 相關文件

- [overlay.md](./overlay.md) — OverlayFS 測試
- [container.md](./container.md) — 容器測試
- [namespace.md](../../kernel/src/namespace.md) — 命名空間
