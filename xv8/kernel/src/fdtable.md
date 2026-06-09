# fd 表管理 — fdtable.rs

## 概述

`fdtable.rs` 實作 process 層級的檔案描述符表管理。xv8 的每個 process 維護一個獨立的 fd 表，從 fd 編號到 `FdEntry` 的映射。Linux 風格的 fd 表實作，支援 close-on-exec 旗標。

## 資料結構

```rust
const NOFILE: usize = 128;  // 每 process 最大 fd 數

pub struct Fdtable {
    pub files: SpinLock<[Option<File>; NOFILE]>,
}

pub struct FdEntry {
    pub file: Option<File>,
    pub cloexec: bool,
}
```

`fdtable.rs` 提供 `fdalloc` 演算法：從最低可用編號開始尋找空閒 fd 槽位，避免過度利用高編號（類似 Linux 的 fd 分配策略）。

## 相關文件

- [sysfile 文件](sysfile.md)
- [file 文件](file.md)
- [proc 文件](proc.md)
