# OverlayFS 模組 — overlay.rs

## 理論背景

OverlayFS (疊合檔案系統) 是一種聯合掛載檔案系統 (union mount filesystem)，將兩個或多個目錄疊合成一個單一目錄樹。上層 (upper) 為可寫層，下層 (lower) 為唯讀層。讀取操作先查上層再查下層，寫入操作僅影響上層。

OverlayFS 的設計源自 BSD 的 union mount、Sun Microsystems 的 Translucent File System、以及 Plan 9 的 union directories。Docker 使用 OverlayFS 作為預設儲存驅動程式，實現容器映像的分層管理。

## xv8 實作

### 資料結構

```rust
pub struct OverlayEntry {
    pub mount_id: u64,
    pub lower: Inode,
    pub upper: Inode,
    pub merged: Inode,
}
```

全域掛載表 `OVERLAY_TABLE` 追蹤所有 Overlay 掛載點。

### overlay_mount

掛載流程：
1. 解析三個路徑 (lower, upper, merged) 的 Inode
2. 驗證 lower 與 upper 皆為目錄
3. 建立 `OverlayEntry`
4. 註冊 merged inode 的回呼

### resolve_inner 整合

在 `fs.rs` 的 `resolve_inner` 中，當解析到 merged 目錄時，疊合 lookup 邏輯自動啟用。此為無縫整合 ── 使用者空間不需要知道檔案實際儲存位置。

### overlay_create

在上層目錄建立新檔案，用於容器初始化的自動上層檔案建立。

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 148 | `overlay_mount` | `(lower: *const u8, upper: *const u8, merged: *const u8)` |
| 149 | `overlay_umount` | `(merged: *const u8)` |

## 相關文件

- [Wiki: OverlayFS](../../../_wiki/OverlayFS.md)
- [Wiki: 容器](../../../_wiki/Container.md)
- [FS 模組文件](fs.md)
- [syscall 文件](syscall.md)
