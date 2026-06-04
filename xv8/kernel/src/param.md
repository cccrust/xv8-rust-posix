# 核心參數 — param.rs

定義 xv8 核心的各種系統限制和配置參數。

## 程序相關參數

```rust
/// 最大 CPU 數量
pub const NCPU: usize = 8;

/// 最大程序數量
pub const NPROC: usize = 64;
```

## 檔案相關參數

```rust
/// 每程序最大開啟檔案數
pub const NOFILE: usize = 256;

/// 系統最大開啟檔案數
pub const NFILE: usize = 100;

/// 最大 inode 快取數量
pub const NINODE: usize = 50;

/// 最大裝置號
pub const NDEV: usize = 10;

/// 根磁碟裝置號
pub const ROOTDEV: u32 = 1;
```

## 網路相關參數

```rust
/// UDP 通訊端最大數量
pub const NSOCKET: usize = 16;

/// Ping 通訊端最大數量
pub const NPING: usize = 16;

/// TCP 通訊端最大數量
pub const NTCP: usize = 16;
```

## 記憶體相關參數

```rust
/// 使用者堆疊頁數
pub const USERSTACK: usize = 4;

/// 程序最大 mmap 區域數
pub const MMAP_REGIONS: usize = 16;

/// mmap 起始位址（低於 trapframe）
pub const MMAP_BASE: usize = TRAPFRAME - (256 * 1024 * 1024);
```

## 核心堆疊

```rust
/// 除錯模式：8 頁（除錯資訊需要更多空間）
#[cfg(debug_assertions)]
pub const NKSTACK_PAGES: usize = 8;

/// 發布模式：1 頁
#[cfg(not(debug_assertions))]
pub const NKSTACK_PAGES: usize = 1;
```

## 檔案系統參數

```rust
/// 最大路徑名稱長度
pub const MAXPATH: usize = 128;

/// exec 最大參數數量
pub const MAXARG: usize = 32;

/// 日誌區塊數（日誌交易大小限制）
pub const LOGBLOCKS: usize = MAXOPBLOCKS * 3;  // 30

/// 區塊緩衝區數量
pub const NBUF: usize = MAXOPBLOCKS * 3;  // 30

/// 任何 FS 操作最大寫入區塊數
pub const MAXOPBLOCKS: usize = 10;
```

## 參數計算

```
LOGBLOCKS = MAXOPBLOCKS * 3 = 30
NBUF = MAXOPBLOCKS * 3 = 30

意義：
- 每個 FS 操作最多修改 10 個區塊
- 日誌需要容納 3 倍的交易大小以確保安全
- 緩衝區數量與日誌大小匹配
```

## 環境變數相關

見 `env` 系統呼叫和 `xv8rust/xv8-libc-compat` 中的實現。

## 相關主題

- [[proc]]：程序管理
- [[fs]]：檔案系統
- [[vm]]：虛擬記憶體
- [[memlayout]]：記憶體佈局