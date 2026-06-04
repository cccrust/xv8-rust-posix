# 檔案系統映像建立工具 — mkfs

mkfs 是一個主機端工具，用於建立 xv8 可開機的檔案系統映像檔。

## 功能

1. 在主機上建立一個空的檔案系統映像檔
2. 格式化為 xv8 相容的日誌結構檔案系統
3. 可選地將二進位檔案嵌入到映像檔中

## 使用方式

```bash
mkfs <fs.img> [files...]
```

## 磁碟佈局

```
區塊 0                 區塊 1         區塊 2                ...
┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│  Boot Block  │ Super Block │   Log 區塊  │  Inode 區塊 │ Bitmap 區塊 │
│   (未使用)   │  (魔數、尺寸) │  (日誌+頭)   │   (inode)   │  (空閒區塊) │
└─────────────┴─────────────┴─────────────┴─────────────┴─────────────┘
                                                    │
                                                    ▼
                      ┌─────────────────────────────────────────────┐
                      │              資料區塊                          │
                      │  (檔案內容、直接區塊、間接區塊)               │
                      └─────────────────────────────────────────────┘
```

## 磁碟參數

```rust
const FSSIZE: u32 = 32768;        // 32768 區塊 = 32 MB
const BSIZE: u32 = 1024;          // 1 KB 區塊大小
const NDIRECT: u32 = 12;           // 直接區塊指標數量
const NINDIRECT: u32 = 256;        // 間接區塊指標數量 (1024/4)
const MAXFILE: u32 = NDIRECT + NINDIRECT;  // 最大檔案大小 (268 區塊)
const NINODES: u32 = 200;          // inode 數量
```

## 超級區塊結構

```rust
struct SuperBlock {
    magic: u32,       // 0x10203040 (驗證檔案系統)
    size: u32,        // 檔案系統總大小 (區塊數)
    nblocks: u32,     // 資料區塊數量
    ninodes: u32,     // inode 數量
    nlogs: u32,       // 日誌區塊數量
    logstart: u32,    // 日誌起始區塊
    inodestart: u32,  // inode 區塊起始
    bmapstart: u32,   // 區塊位圖起始
}
```

## 區塊計算

```rust
const LOGBLOCKS: u32 = MAXOPBLOCKS * 3;     // 30 區塊
const NLOG: u32 = LOGBLOCKS + 1;           // 31 區塊 (含頭)
const NBITMAP: u32 = FSSIZE / BPB + 1;      // 區塊位圖
const NINODEBLOCKS: u32 = NINODES / IPB + 1; // inode 區塊
const NMETA: u32 = NLOG + NINODEBLOCKS + NBITMAP + 2;  // 中繼區塊
const NBLOCKS: u32 = FSSIZE - NMETA;        // 資料區塊
```

## 配置值

| 參數 | 值 | 說明 |
|------|-----|------|
| FSSIZE | 32768 | 總區塊數 |
| BSIZE | 1024 | 區塊大小 |
| NMETA | ~35 | 中繼區塊數 |
| NBLOCKS | ~32733 | 資料區塊數 |
| NINODES | 200 | inode 數量 |
| NDIRECT | 12 | 直接區塊 |
| NINDIRECT | 256 | 間接區塊 |
| MAXFILE | 268 | 最大檔案大小 (區塊) |

## 建立流程

```rust
fn main() {
    // 1. 建立超級區塊
    let sb = SuperBlock {
        magic: FSMAGIC,
        size: FSSIZE,
        nblocks: NBLOCKS,
        ninodes: NINODES,
        nlogs: NLOG,
        logstart: 2,
        inodestart: 2 + NLOG,
        bmapstart: 2 + NLOG + NINODEBLOCKS,
    };

    // 2. 將整個映像檔清零
    for i in 0..FSSIZE {
        write_sector(&file, i, &ZEROS);
    }

    // 3. 寫入超級區塊
    write_sector(&file, 1, &sb);

    // 4. 建立根目錄
    let rootino = allocate_inode(&file, DIRECTORY, &mut free_inode);

    // 5. 建立 . 和 .. 目錄項
    append_inode(&file, &mut free_block, rootino, ".".as_bytes());
    append_inode(&file, &mut free_block, rootino, "..".as_bytes());

    // 6. 可選：嵌入二進位檔案
    for path in &args[2..] {
        let inum = allocate_inode(&file, FILE, &mut free_inode);
        append_inode(&file, &mut free_block, rootino, file_entry);
        append_inode(&file, &mut free_block, inum, file_content);
    }

    // 7. 更新根目錄大小
    update_root_size();

    // 8. 設定區塊位圖
    allocate_block(&file, free_block, bmapstart);
}
```

## 區塊配置函式

```rust
fn allocate_block(file: &File, used: u32, bmapstart: u32) {
    // 遍歷每個位圖區塊
    for block in 0..NBITMAP {
        let start = block * BPB;
        if start >= used {
            break;
        }

        // 標記已使用的區塊
        let end = (start + BPB).min(used);
        for i in start..end {
            buf[i / 8] |= 1 << (i % 8);
        }

        write_sector(file, bmapstart + block, &buf);
    }
}
```

## inode 配置

```rust
fn allocate_inode(file: &File, type: InodeType, free_inode: &mut u32) -> u32 {
    let inum = *free_inode;
    *free_inode += 1;

    let din = DiskInode::new(type);
    din.nlink = 1;
    din.size = 0;

    write_inode(file, inum, &din);
    inum
}
```

## 資料附加到 inode

```rust
fn append_inode(file: &File, free_block: &mut u32, inum: u32, data: &[u8]) {
    let mut din = read_inode(file, inum);
    let mut offset = din.size;

    while !data.is_empty() {
        let fbn = offset / BSIZE;

        // 取得或配置區塊位址
        let x = if fbn < NDIRECT {
            if din.addrs[fbn as usize] == 0 {
                din.addrs[fbn as usize] = *free_block;
                *free_block += 1;
            }
            din.addrs[fbn as usize]
        } else {
            // 間接區塊處理...
        };

        // 讀取、修改、寫回
        read_sector(file, x, &mut buf);
        buf[block_offset..].copy_from_slice(&data[..n]);
        write_sector(file, x, &buf);

        offset += n as u32;
        data = &data[n..];
    }

    din.size = offset;
    write_inode(file, inum, &din);
}
```

## 與 xv8 核心的整合

```bash
# 建立檔案系統映像
./mkfs.sh

# 或手動
cd mkfs && cargo run --release -- ../fs.img ../user/bin/* ../user/testbin/*
```

mkfs.sh 會：
1. 編譯 mkfs 工具
2. 建立 fs.img
3. 嵌入所有使用者程式和測試二進位檔

## 檔案系統驗證

xv8 啟動時會驗證魔數：

```rust
if sb.magic != FSMAGIC {
    panic("invalid file system");
}
```

## 限制

- 檔案系統大小固定為 32MB
- 單一檔案最大 268 區塊（約 268KB）
- 根目錄的 `.` 和 `..` 在建立後大小固定

## 相關主題

- [[fs]]：xv8 核心的檔案系統實現
- [[Boot]]：啟動流程