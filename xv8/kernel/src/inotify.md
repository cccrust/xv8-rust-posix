# inotify 模組 — inotify.rs

## 理論背景

inotify (inode notify) 是 Linux 2.6.13 引入的檔案系統事件監控機制，讓應用程式可以監控檔案與目錄的變更事件（建立、刪除、修改、移動等）。inotify 取代了舊的 `dnotify` 機制，提供更豐富的事件類型和更簡單的程式設計介面。

## xv8 實作

### 事件類型

```rust
pub const IN_ACCESS: u32        = 0x00000001;  // 檔案被存取
pub const IN_MODIFY: u32        = 0x00000002;  // 檔案被修改
pub const IN_ATTRIB: u32        = 0x00000004;  // 中繼資料變更
pub const IN_CLOSE_WRITE: u32   = 0x00000008;  // 可寫 fd 關閉
pub const IN_CLOSE_NOWRITE: u32 = 0x00000010;  // 唯讀 fd 關閉
pub const IN_OPEN: u32          = 0x00000020;  // 檔案被打開
pub const IN_MOVED_FROM: u32    = 0x00000040;  // 移出目錄
pub const IN_MOVED_TO: u32      = 0x00000080;  // 移入目錄
pub const IN_CREATE: u32        = 0x00000100;  // 建立檔案/目錄
pub const IN_DELETE: u32        = 0x00000200;  // 刪除檔案/目錄
pub const IN_DELETE_SELF: u32   = 0x00000400;  // 自身被刪除
pub const IN_MOVE_SELF: u32     = 0x00000800;  // 自身被移動
pub const IN_ALL_EVENTS: u32    = 0x00000fff;  // 所有事件
```

### 資料結構

```rust
pub struct InotifyEvent {
    pub wd: i32,           // watch descriptor
    pub mask: u32,         // 事件遮罩
    pub cookie: u32,       // rename cookie (配對 MOVED_FROM/MOVED_TO)
    pub len: u32,          // name 長度
    pub name: [u8; 16],    // 事件關聯的檔名
}
```

### 操作

| 操作 | 行為 |
|------|------|
| `read()` | 從事件佇列取出 `InotifyEvent` |
| `write()` | 傳回 `BadDescriptor` |
| `INOTIFY_ADD_WATCH` | 加入監控目錄 |
| `INOTIFY_RM_WATCH` | 移除監控 |
| `poll()` | 事件佇列非空時可讀 |

### 內部通知機制

`notify()` 函數在檔案系統操作時被呼叫，觸發對應的 inotify 事件：

```rust
pub fn notify(dev: u32, inum: u32, mask: u32, cookie: u32, name: &str);
```

此函數遍歷所有 inotify 監控項，若匹配則將事件推入對應的事件佇列。

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 39 | `inotify_init1` | `(flags: u32)` |
| 40 | `inotify_add_watch` | `(fd: i32, path: *const u8, mask: u32)` |
| 41 | `inotify_rm_watch` | `(fd: i32, wd: i32)` |

## 相關文件

- [syscall 文件](syscall.md)
- [file 文件](file.md)
- [fs 文件](fs.md)
