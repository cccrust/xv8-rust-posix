# memfd 模組 — memfd.rs

## 理論背景

memfd (memory file descriptor) 是 Linux 3.17 引入的機制，建立在核心記憶體中的匿名檔案。memfd 可以用於：

- 不需要實際磁碟空間的暫存檔案
- 程序間通訊 (與 fd 傳遞配合)
- 秘密管理 (記憶體永遠不會 swap 到磁碟)

## xv8 實作

### 資料結構

```rust
struct MemFdState {
    data: Vec<u8>,        // 記憶體緩衝區
}
```

全域 `MEMFD_TABLE` 管理所有 memfd 實例。

### 操作

| 操作 | 行為 |
|------|------|
| `read(offset, buf)` | 從指定偏移讀取資料 |
| `write(offset, buf)` | 從指定偏移寫入資料 |
| `lseek` | 支援 SEEK_SET, SEEK_CUR, SEEK_END |

與在磁碟上的 `Inode` 不同，memfd 所有資料完全在記憶體中，支援基本的 `lseek` 操作。

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 33 | `memfd_create` | `(name: *const u8, flags: u32)` |

## 相關文件

- [syscall 文件](syscall.md)
- [file 文件](file.md)
