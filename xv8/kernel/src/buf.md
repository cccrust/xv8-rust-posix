# 緩衝區塊快取 — buf.rs

緩衝區塊快取將磁碟區塊暫存在記憶體中，減少磁碟 I/O 並提供多程序同步點。

## 設計目標

1. **減少磁碟讀取**：頻繁存取的區塊保持在記憶體
2. **寫入合併**：多次修改只在最後一次性寫入磁碟
3. **同步**：確保同一區塊同時只有一個程序修改

## 資料結構

```rust
struct BCache {
    // 保護中繼資料（ref_count, LRU 鏈表等）
    inner: SpinLock<BCacheInner>,
    // 每個緩衝區的睡眠鎖（保護實際資料）
    bufs: [SleepLock<BufData>; NBUF],
}

struct BCacheInner {
    meta: [BufMeta; NBUF],  // 緩衝區中繼資料
    head: usize,             // LRU 鏈表頭
}

struct BufMeta {
    valid: bool,            // 資料是否有效
    disk: bool,             // 是否需要寫回磁碟
    dev: u32,               // 裝置號
    block_no: u32,          // 區塊編號
    ref_count: u32,         // 參考計數
    prev: usize,            // LRU 鏈表
    next: usize,
}

struct BufData {
    data: [u8; BSIZE],      // 1024 位元組區塊資料
}
```

## LRU 替換策略

使用雙向鏈表實現 LRU（Least Recently Used）：

```rust
// 初始化：形成循環鏈表
// head -> 0 -> 1 -> ... -> NBUF-1 -> head
inner.head = 0;
for i in 0..NBUF {
    meta[i].prev = if i == 0 { NBUF - 1 } else { i - 1 };
    meta[i].next = if i == NBUF - 1 { 0 } else { i + 1 };
}
```

## 緩衝區獲取

```rust
pub fn get(&self, dev: u32, block_no: u32) -> Buf<'_> {
    let mut inner = self.inner.lock();

    // 1. 查找是否已快取
    for i in 0..NBUF {
        if meta.dev == dev && meta.block_no == block_no {
            meta.ref_count += 1;
            drop(inner);
            return Buf { id: i, guard: self.bufs[i].lock() };
        }
    }

    // 2. 未命中，從 LRU 末端回收
    let mut i = inner.meta[inner.head].prev;
    loop {
        if i == inner.head {
            panic!("bcache get no buffers");
        }

        if meta.ref_count == 0 {
            meta.dev = dev;
            meta.block_no = block_no;
            meta.valid = false;
            meta.ref_count += 1;
            drop(inner);
            return Buf { id: i, guard: self.bufs[i].lock() };
        }
        i = meta[i].prev;
    }
}
```

## 讀取區塊

```rust
pub fn read(&self, dev: u32, block_no: u32) -> Buf<'_> {
    let mut buf = self.get(dev, block_no);

    let valid = {
        let lock = self.inner.lock();
        lock.meta[buf.id].valid
    };

    if !valid {
        // 從磁碟讀取
        virtio_disk::rw(&mut buf, false);
        self.inner.lock().meta[buf.id].valid = true;
    }

    buf
}
```

## 寫入區塊

```rust
pub fn write(&self, buf: &mut Buf<'_>) {
    // buf 已鎖定（SleepLock），直接寫入磁碟
    virtio_disk::rw(buf, true);
}
```

## 釋放緩衝區

```rust
pub fn release(&self, buf: Buf<'_>) {
    let id = buf.id;
    drop(buf);  // 釋放 SleepLock

    let mut inner = self.inner.lock();
    inner.meta[id].ref_count -= 1;

    if meta.ref_count == 0 {
        // 移動到 LRU 鏈表頭部（最近使用）
        let next = meta[id].next;
        let prev = meta[id].prev;
        meta[next].prev = prev;
        meta[prev].next = next;

        let head = inner.head;
        let first = meta[head].next;
        meta[id].next = first;
        meta[id].prev = head;
        meta[first].prev = id;
        meta[head].next = id;
    }
}
```

## 固定/解除固定

```rust
pub fn pin(&self, buf: &Buf<'_>) {
    // 增加 ref_count，防止被回收
    self.inner.lock().meta[buf.id].ref_count += 1;
}

pub fn unpin(&self, buf: &Buf<'_>) {
    self.inner.lock().meta[buf.id].ref_count -= 1;
}
```

## 雙層鎖定策略

```
SpinLock (bcache.inner)          保護：
- 區塊是否在快取中
- ref_count
- LRU 鏈表

SleepLock (bufs[i])              保護：
- 實際區塊資料
- 允許長時間持有（可睡眠）
- 在 I/O 期間保持鎖定
```

## 與日誌系統的整合

寫入時呼叫 `log::write()` 而非直接呼叫 `BCACHE::write()`，以確保寫入被記錄在日誌中。

## 相關主題

- [[log]]：預寫式日誌
- [[fs]]：檔案系統
- [[virtio_disk]]：磁碟驅動