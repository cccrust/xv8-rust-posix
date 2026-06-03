# File-System（檔案系統）

xv8 實現了一個日誌結構檔案系統（Log-structured File System），具有寫入時日志（Write-Ahead Logging）來確保在系統崩潰時能恢復一致性。

## 整體架構

xv8 檔案系統分為幾層：

```
┌──────────────────────────────────────┐
│           系統呼叫層                   │
│  (open, read, write, close, etc.)     │
├──────────────────────────────────────┤
│           虛擬檔案系統層 (VFS)         │
│     (統一的檔案/目錄/連結介面)          │
├──────────────────────────────────────┤
│           區塊層                       │
│   (緩衝區快取、日誌管理)               │
├──────────────────────────────────────┤
│         裝置驅動層                     │
│        (VirtIO Disk)                  │
└──────────────────────────────────────┘
```

## 磁碟佈局

xv8 的磁碟映像（fs.img）結構：

```
┌─────────┬─────────────┬────────────────────┬──────────────────────────┐
│ Boot    │ Superblock  │     Log            │     Data blocks          │
│ block   │ (1 block)   │  (2000 blocks)     │  (剩余空间)               │
└─────────┴─────────────┴────────────────────┴──────────────────────────┘
```

- **Boot block**：開機區塊（目前未使用）
- **Superblock**：檔案系統中繼資料（大小、空閒區塊、inode 表位置等）
- **Log**：日誌區域，用於 Write-Aided Logging
- **Data blocks**：實際檔案資料存放處

## Superblock 結構

Superblock 包含檔案系統的全局資訊：

```rust
pub struct Superblock {
    pub magic: u32,           // 魔術數 0x12345678，驗證檔案系統
    pub size: u32,            // 檔案系統總區塊數
    pub nblocks: u32,         // 資料區塊數量
    pub ninodes: u32,         // inode 數量
    pub inodestart: u32,      // inode 表起始區塊
    pub bmapstart: u32,       // 位元圖起始區塊
    pub logstart: u32,        // 日誌起始區塊
    pub logsize: u32,         // 日誌區塊數
}
```

## Inode 結構

inode 是 xv8 檔案系統的核心，每個檔案或目錄都有一個 inode：

```rust
pub struct Dinode {
    pub type_: u16,           // 類型：檔案、目錄、裝置
    pub major: u16,           // 主設備號（對於裝置檔案）
    pub minor: u16,           // 副設備號
    pub nlink: u16,           // 硬連結數
    pub size: u32,            // 檔案大小（位元組）
    pub addrs: [u32; 27],     // 直接、間接區塊指標
    pub timestamp: u64,       // 時間戳記
}
```

一個 inode 最多可以有 27 個區塊指標：
- 前 13 個是直接區塊指標（每個 4KB，理論最大 52KB）
- 第 14 個是一級間接區塊指標（可額外索引 1024 個區塊，共約 4MB）
- 第 15 個是二級間接區塊指標（可索引 1024×1024 個區塊）

## 目錄格式

目錄在 xv8 中是一種特殊的檔案。其內容是目錄條目的線性陣列：

```rust
pub struct Dirent {
    pub inum: u16,            // inode 編號
    pub name: [u8; 28],       // 檔案名稱（最多 27 字元 + null）
}
```

讀取目錄時，核心解析這些 Dirent 條目。`getdents` 系統呼叫以 POSIX 相容格式返回目錄條目。

## Write-Aided Logging（WAL）

WAL 是 xv8 檔案系統一致性的關鍵。傳統的檔案系統在系統崩潰時可能損壞（目錄更新了但內容區塊還沒寫入）。WAL 確保這些操作是原子的。

### 日誌工作流程

1. **Write**：在日誌區域寫入交易記錄
   - 包含所有要更新的區塊（superblock 副本、inode、資料區塊）
   - 交易以一個 "commit" 記錄結尾

2. **Install**：將日誌中的修改安裝到最終位置
   - 讀取日誌中的資料，寫入實際的磁碟區塊

3. **Truncate**：截斷日誌，釋放空間

### 交易格式

```
┌────────────┬──────────┬─────────────┬────────────┐
│ Header     │ Block 1  │   Block 2   │  Commit    │
│ (4 bytes)  │          │             │  (4 bytes) │
└────────────┴──────────┴─────────────┴────────────┘
```

Header 包含交易中包含的區塊數量。每個交易的總大小寫在 Header 中。

## 緩衝區快取

`buf.rs` 實現了一個簡單的緩衝區快取：

- 磁碟區塊被快取在記憶體中以減少 I/O
- 每個緩衝區有一把 sleeplock，允許程序在 I/O 期間阻塞
- 緩衝區是基於 LRU（最少使用）替換的簡單實現

## 路徑解析

路徑解析從根目錄或程序目前目錄開始，逐步解析每個路徑組成部分：

```
path = "/home/user/file.txt"
分解為：["home", "user", "file.txt"]

1. 從根 inode ("/") 讀取目錄內容
2. 找到 "home" 條目，取得其 inode
3. 從 "home" inode 讀取目錄內容
4. 找到 "user" 條目，取得其 inode
5. 從 "user" inode 讀取目錄內容
6. 找到 "file.txt" 條目
7. 返回該 inode
```

## 檔案描述符

每個程序有一個檔案描述符表（最多 16 個項目），每個項目指向一個 `File` 結構：

```rust
pub struct File {
    pub inode: Arc<dyn INode>,   // 底層 inode
    pub offset: u64,              // 檔案讀寫偏移
    pub kind: FileKind,          // 類型：Fd, Pipe, Device
}
```

打開檔案時，核心分配一個閒置的 fd 槽位並初始化 File 結構。

## 連結（Hard Link）與符號連結

- **Hard Link**：多個目錄條目指向同一個 inode。`nlink` 計數器追蹤引用數量。當 `nlink` 降到 0 且沒有程序開啟該檔案時，回收 inode 和資料區塊。
- **Symbolic Link**：一種特殊類型的檔案，內容是另一個路徑名稱。解析時會跟隨連結。

## Mknod 與裝置檔案

`mknod` 系統呼叫建立裝置特殊檔案：

```rust
sys_mknod(path: *const u8, major: u32, minor: u32) -> isize
```

裝置檔案是一種特殊的 inode，其 `type_` 為 T_DEV。讀寫裝置檔案會路由到對應的字元裝置或區塊裝置驅動程式。

## 同步操作

`sync` 系統呼叫確保所有緩衝區的修改寫入磁碟：
- 標記所有「dirty」緩衝區
- 等待所有 I/O 完成

## 相關主題

- [[Device-Drivers]]：VirtIO 磁碟驅動
- [[Process]]：程序的檔案描述符管理
- [[Syscall]]：相關的系統呼叫（open, read, write, mkdir 等）