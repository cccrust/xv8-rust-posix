# VirtIO 磁碟驅動 — virtio_disk.rs

VirtIO 是一種高效的虛擬化 I/O 框架，用於 QEMU 提供的虛擬磁碟。

## QEMU 配置

```
QEMU virt machine
         │
         │ VirtIO MMIO
         │
         ▼
    VirtIO Block Device
    - 位置: 0x10001000
    - IRQ: 1
    - 容量: 約 128MB
```

## VirtIO MMIO 寄存器

```rust
const VIRTIO_MMIO_MAGIC_VALUE: u32 = 0x000;  // "virt" (0x74726976)
const VIRTIO_MMIO_VERSION: u32 = 0x004;      // 版本 (應為 2)
const VIRTIO_MMIO_DEVICE_ID: u32 = 0x008;     // 裝置類型 (2=磁碟)
const VIRTIO_MMIO_DRIVER_FEATURES: u32 = 0x020;
const VIRTIO_MMIO_QUEUE_SEL: u32 = 0x030;    // 選擇佇列
const VIRTIO_MMIO_QUEUE_NUM: u32 = 0x038;    // 佇列大小
const VIRTIO_MMIO_QUEUE_READY: u32 = 0x044;  // 佇列就緒
const VIRTIO_MMIO_QUEUE_NOTIFY: u32 = 0x050; // 通知裝置
const VIRTIO_MMIO_INTERRUPT_STATUS: u32 = 0x060;
const VIRTIO_MMIO_STATUS: u32 = 0x070;       // 狀態
```

## VirtIO 描述符結構

```rust
struct VirtqDesc {
    addr: u64,      // 緩衝區位址
    len: u32,       // 緩衝區長度
    flags: u16,     // VRING_DESC_F_NEXT, VRING_DESC_F_WRITE
    next: u16,      // 下一個描述符鏈
}

struct VirtqAvail {
    flags: u16,
    idx: u16,       // 驅動程式寫入下一個可用環項目
    ring: [u16; NUM],  // 描述符索引
}

struct VirtqUsed {
    flags: u16,
    idx: u16,       // 裝置寫入已完成項目
    ring: [VirtqUsedElem; NUM],
}
```

## 佇列結構

```
可用環 (Avail):           已用環 (Used):
┌──────────────┐          ┌──────────────┐
│ ring[0..7]  │          │ ring[0..7]  │
│ (描述符索引) │          │ (已完成)     │
└──────────────┘          └──────────────┘
     ▲                         ▲
     │                         │
 驅動程式寫入                 裝置寫入
```

## 磁碟請求格式

```rust
struct BlockReq {
    type: u32,     // VIRTIO_BLK_T_IN (0) 或 VIRTIO_BLK_T_OUT (1)
    reserved: u32,
    sector: u64,   // 區塊編號 (512 位元組為單位)
}
```

一個請求由三個描述符組成：
1. BlockReq 結構體（讀取）
2. 資料緩衝區（讀取/寫入）
3. 狀態位元組（寫入）

## 初始化

```rust
pub unsafe fn init() {
    let mut disk = VIRTIO_DISK.lock();

    // 檢查 VirtIO magic
    assert_eq!(read_reg(VIRTIO_MMIO_MAGIC_VALUE), 0x74726976);

    // 重置
    write_reg(VIRTIO_MMIO_STATUS, 0);
    write_reg(VIRTIO_MMIO_STATUS, VIRTIO_CONFIG_S_ACKNOWLEDGE);
    write_reg(VIRTIO_MMIO_STATUS, VIRTIO_CONFIG_S_DRIVER);
}
```

## 讀取/寫入請求

```rust
pub fn rw(buf: &mut Buf, write: bool) {
    let mut disk = VIRTIO_DISK.lock();

    // 等待直到有可用描述符
    while !disk.free.iter().any(|x: &bool| *x) {
        drop(disk);
        proc::sleep(Channel::Buffer(&VIRTIO_DISK as *const _ as usize), VIRTIO_DISK.lock());
        disk = VIRTIO_DISK.lock();
    }

    let i = disk.free.iter().position(|x| *x).unwrap();
    disk.free[i] = false;

    // 設定描述符鏈
    disk.desc[i * 3 + 0].addr = &disk.info[i] as *const _ as u64;
    disk.desc[i * 3 + 0].len = size_of::<BlockReq>() as u32;
    disk.desc[i * 3 + 0].flags = VRING_DESC_F_NEXT;
    disk.desc[i * 3 + 0].next = (i * 3 + 1) as u16;

    // ... 設定資料和狀態描述符 ...

    // 更新可用環
    let idx = disk.avail.idx as usize % NUM;
    disk.avail.ring[idx] = (i * 3) as u16;
    disk.avail.idx += 1;

    // 通知 VirtIO
    write_reg(VIRTIO_MMIO_QUEUE_NOTIFY, 0);

    // 睡眠等待完成
    while disk.info[i].status == 0 {
        drop(disk);
        proc::sleep(Channel::Buffer(&disk.info[i] as *const _ as usize), VIRTIO_DISK.lock());
        disk = VIRTIO_DISK.lock();
    }
}
```

## 中斷處理

```rust
pub fn handle_interrupt() {
    let mut disk = VIRTIO_DISK.lock();

    let status = read_reg(VIRTIO_MMIO_INTERRUPT_STATUS);
    write_reg(VIRTIO_MMIO_INTERRUPT_ACK, status);

    // 處理已完成的描述符
    while disk.used.idx != disk.used_idx {
        let used_elem = disk.used.ring[disk.used_idx as usize % NUM];
        let i = used_elem.id as usize;

        disk.info[i].status = 1;  // 完成
        disk.free[i] = true;

        proc::wakeup(Channel::Buffer(&disk.info[i] as *const _ as usize));

        disk.used_idx += 1;
    }
}
```

## 與 Buffer Cache 的整合

```rust
pub fn read(dev: u32, block_no: u32) -> Buf<'_> {
    let mut buf = BCACHE.get(dev, block_no);

    if !buf.valid {
        // 從磁碟讀取
        virtio_disk::rw(&mut buf, false);
    }

    buf
}
```

## 相關主題

- [[buf]]：緩衝區塊快取
- [[fs]]：檔案系統
- [[memlayout]]：記憶體映射