# 檔案系統 — fs.rs

xv8 實現了一個簡單的磁碟檔案系統，使用預寫式日誌（Write-Ahead Logging）確保一致性。

## 磁碟佈局

```
┌─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ Boot    │Super    │ Log     │ Inode   │ Bitmap  │ Data    │
│ Block   │ Block   │ Blocks  │ Blocks  │ Blocks  │ Blocks  │
│ (1)     │ (1)     │ (n)     │ (n)     │ (n)     │ (rest)  │
└─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘
         ↑
    block 1
```

## 磁碟區塊結構

```rust
const BSIZE: usize = 1024;  // 區塊大小
const NDIRECT: usize = 12;  // 直接區塊指標數量
const NINDIRECT: usize = 256;  // 間接區塊指標數量 (1024 / 4)
const MAXFILE: usize = NDIRECT + NINDIRECT;  // 最大檔案大小 (268 blocks)
const DIRSIZE: usize = 14;  // 目錄項名稱大小
```

## 超級區塊 (SuperBlock)

```rust
struct SuperBlock {
    magic: u32,        // FS_MAGIC = 0x10203040
    size: u32,         // 檔案系統大小 (區塊數)
    nblocks: u32,      // 資料區塊數量
    ninodes: u32,      // inode 數量
    nlogs: u32,        // 日誌區塊數量
    logstart: u32,     // 日誌起始區塊
    inodestart: u32,   // inode 區塊起始
    bmapstart: u32,    // 區塊位圖起始
}
```

## inode 結構

磁碟上的 inode（`DiskInode`）：
```rust
struct DiskInode {
    type: InodeType,   // 類型 (File/Directory/Device/SymLink/Fifo)
    major: u16,        // 主裝置號
    minor: u16,        // 次裝置號
    nlink: u16,        // 連結數
    size: u32,         // 檔案大小
    addrs: [u32; 13],  // 區塊位址 (12 direct + 1 indirect)
}
```

記憶體中的 inode（`Inode`）：
```rust
struct Inode {
    id: usize,         // inode 表索引
    dev: u32,          // 裝置號
    inum: u32,         // inode 編號
}

struct InodeInner {
    valid: bool,
    type: InodeType,
    major: u16,
    minor: u16,
    nlink: u16,
    uid: u16,
    gid: u16,
    size: u32,
    addrs: [u32; 13],
}
```

## inode 表

```rust
const NINODE: usize = 50;

pub static INODE_TABLE: InodeTable = InodeTable::new();

struct InodeTable {
    meta: SpinLock<[InodeMeta; NINODE]>,   // 參考計數、元資料
    inner: [SleepLock<InodeInner>; NINODE], // inode 資料
}

struct InodeMeta {
    dev: u32,
    inum: u32,
    ref: u32,  // 參考計數
}
```

## inode 操作

### 分配 inode

```rust
pub fn alloc(dev: u32, type: InodeType) -> Result<Self, FsError> {
    for inum in 1..sb.ninodes {
        let buf = BCACHE.read(dev, sb.inodestart + inum / IPB);
        let dinode = unsafe { DiskInode::from_buf(&mut buf, inum) };

        if dinode.type == InodeType::Free {
            dinode.type = type;  // 標記為已使用
            log::write(&buf);
            BCACHE.release(buf);
            return Self::get(dev, inum);
        }
        BCACHE.release(buf);
    }
    err!(FsError::OutOfInode)
}
```

### 讀取 inode

```rust
pub fn lock(&self) -> SleepLockGuard<'static, InodeInner> {
    let mut inner = INODE_TABLE.inner[self.id].lock();

    if !inner.valid {
        // 從磁碟讀取
        let buf = BCACHE.read(self.dev, sb.inodestart + (self.inum / IPB));
        let dinode = unsafe { DiskInode::from_buf(&mut buf, self.inum) };

        inner.type = dinode.type;
        inner.size = dinode.size;
        inner.addrs.copy_from_slice(&dinode.addrs);
        // ...

        BCACHE.release(buf);
        inner.valid = true;
    }
    inner
}
```

### 釋放 inode

```rust
pub fn put(mut self) {
    let mut meta = INODE_TABLE.meta.lock();

    if meta[self.id].ref == 1 {
        let mut inner = INODE_TABLE.inner[self.id].lock();

        if inner.valid && inner.nlink == 0 {
            // 沒有連結，截斷並釋放
            self.trunc(&mut inner);
            inner.type = InodeType::Free;
            self.update(&inner);
            inner.valid = false;
        }
    }

    meta[self.id].ref -= 1;
}
```

## 資料區塊配置

```rust
impl Block {
    pub fn alloc(dev: u32) -> Result<Self, FsError> {
        // 掃描區塊位圖找空閒區塊
        for b in (0..sb.size).step_by(BPB) {
            let buf = BCACHE.read(dev, sb.bmapstart + (b / BPB));

            for bi in 0..BPB {
                if buf.data()[bi / 8] & (1 << (bi % 8)) == 0 {
                    // 找到空閒區塊
                    buf.data()[bi / 8] |= 1 << (bi % 8);
                    log::write(&buf);
                    BCACHE.release(buf);

                    // 清零區塊
                    let mut block = Self(b + bi);
                    block.zero(dev);

                    return Ok(block);
                }
            }
            BCACHE.release(buf);
        }
        err!(FsError::OutOfBlock)
    }

    pub fn free(self, dev: u32) {
        // 清除位圖中的位
        let mut buf = BCACHE.read(dev, sb.bmapstart + (self.0 / BPB));
        buf.data()[bi / 8] &= !(1 << (bi % 8));
        log::write(&buf);
    }
}
```

## 檔案讀寫

```rust
pub fn read(&self, inner: &mut SleepLockGuard<'_, InodeInner>,
            offset: u32, dst: &mut [u8], dst_user: bool) -> Result<u32, FsError> {
    if offset > inner.size {
        err!(FsError::Read);
    }

    let n = (inner.size - offset).min(dst.len() as u32);
    let mut total = 0;

    while total < n {
        let addr = log!(self.map(inner, offset / BSIZE as u32))?;
        let buf = BCACHE.read(self.dev, addr);

        let m = (n - total).min(BSIZE as u32 - offset % BSIZE as u32);
        // 複製資料
        // ...

        BCACHE.release(buf);
        total += m;
        offset += m;
    }

    Ok(total)
}

pub fn write(&self, inner: &mut SleepLockGuard<'_, InodeInner>,
             offset: u32, src: &[u8], src_user: bool) -> Result<u32, FsError> {
    let n = src.len() as u32;

    if offset + n > (MAXFILE * BSIZE) as u32 {
        err!(FsError::Write);
    }

    let mut total = 0;
    while total < n {
        let addr = log!(self.map(inner, offset / BSIZE as u32))?;
        let buf = BCACHE.read(self.dev, addr);

        // 複製資料
        log::write(&buf);
        BCACHE.release(buf);

        total += m;
        offset += m;
    }

    if offset > inner.size {
        inner.size = offset;
    }
    self.update(inner);  // 寫回 inode

    Ok(total)
}
```

## 目錄操作

```rust
struct Directory {
    inum: u16,          // inode 編號
    name: [u8; DIRSIZE], // 名稱 (14 bytes)
}

// 查詢目錄項
pub fn lookup(inode: &Inode, inner: &mut SleepLockGuard<'_, InodeInner>,
              name: &str) -> Result<Option<(u32, Inode)>, FsError> {
    for offset in (0..inner.size).step_by(Directory::SIZE) {
        let dir = Self::from_inode(inode, inner, offset)?;

        if dir.inum == 0 {
            continue;
        }

        if dir.is_name_equal(name) {
            return Ok(Some((offset, Inode::get(inode.dev, dir.inum as u32)?)));
        }
    }
    Ok(None)
}

// 建立目錄項
pub fn link(inode: &Inode, inner: &mut SleepLockGuard<'_, InodeInner>,
           name: &str, inum: u16) -> Result<(), FsError> {
    // 檢查是否已存在
    if Self::lookup(inode, inner, name)?.is_some() {
        err!(FsError::Link);
    }

    // 找空閒槽位
    let mut offset = 0;
    while offset < inner.size {
        let dir = Self::from_inode(inode, inner, offset)?;
        if dir.inum == 0 {
            break;
        }
        offset += Directory::SIZE;
    }

    // 寫入新項
    let mut dir = Directory::new_empty();
    dir.set_name(name);
    dir.inum = inum;
    inode.write(inner, offset, dir.as_bytes(), false)?;
    Ok(())
}
```

## 路徑解析

```rust
pub fn resolve(&self) -> Result<Inode, FsError> {
    self.resolve_inner(false).map(|(inode, _)| inode)
}

pub fn resolve_parent(&self) -> Result<(Inode, &'a str), FsError> {
    self.resolve_inner(true)
}

fn resolve_inner(&self, parent: bool) -> Result<(Inode, &'a str), FsError> {
    let mut inode = if self.is_absolute() {
        Inode::get(ROOTDEV, ROOTINO)?
    } else {
        proc::current_proc().data().cwd.dup()
    };

    while let Some((component, rest)) = path.next_component() {
        let mut inner = inode.lock();

        if inner.type != InodeType::Directory {
            inode.unlock_put(inner);
            err!(FsError::Resolve);
        }

        // 最後一個元件？
        if parent && rest.is_empty() {
            inode.unlock(inner);
            return Ok((inode, component));
        }

        match log!(Directory::lookup(&inode, &mut inner, component)) {
            Ok(Some((_, next))) => {
                inode.unlock_put(inner);
                inode = next;
            }
            Ok(None) => {
                inode.unlock_put(inner);
                err!(FsError::Resolve);
            }
            Err(e) => return Err(e),
        }
    }

    if parent {
        inode.put();
        err!(FsError::Resolve);
    }

    Ok((inode, ""))
}
```

## 檔案建立

```rust
pub fn create(path: &Path, type: InodeType, major: u16, minor: u16)
    -> Result<(Self, SleepLockGuard<'static, InodeInner>), FsError> {
    let (parent, name) = try_log!(path.resolve_parent());
    let mut parent_inner = parent.lock();

    // 檢查是否已存在
    if let Ok(Some((_, inode))) = log!(Directory::lookup(&parent, &mut parent_inner, name)) {
        parent.unlock_put(parent_inner);

        let inode_inner = inode.lock();
        if type == InodeType::File
            && (inode_inner.type == InodeType::File || inode_inner.type == InodeType::Device)
        {
            return Ok((inode, inode_inner));  // 覆蓋 existing 檔案
        }
        inode.unlock_put(inode_inner);
        err!(FsError::Create);
    }

    // 分配新 inode
    let inode = log!(Self::alloc(parent.dev, type))?;
    let mut inode_inner = inode.lock();
    inode_inner.major = major;
    inode_inner.nlink = 1;
    inode.update(&inode_inner);

    // 建立目錄的 . 和 .. 項
    if type == InodeType::Directory {
        Directory::link(&inode, &mut inode_inner, ".", inode.inum as u16)?;
        Directory::link(&inode, &mut inode_inner, "..", parent.inum as u16)?;
    }

    // 建立目錄項
    Directory::link(&parent, &mut parent_inner, name, inode.inum as u16)?;

    if type == InodeType::Directory {
        parent_inner.nlink += 1;
        parent.update(&parent_inner);
    }

    parent.unlock_put(parent_inner);
    Ok((inode, inode_inner))
}
```

## 相關主題

- [[File-System]]：通用檔案系統概念
- [[log]]：預寫式日誌
- [[buf]]：區塊緩衝