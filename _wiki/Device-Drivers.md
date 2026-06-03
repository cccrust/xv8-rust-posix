# Device-Drivers（裝置驅動程式）

xv8 的裝置驅動程式提供對硬體的存取，包括 UART（序列埠）、VirtIO（虛擬化磁碟）和 E1000（網路卡）。

## 裝置驅動架構

xv8 採用分層的 I/O 架構：

```
應用程式
   ↓
系統呼叫（read/write/ioctl）
   ↓
虛擬檔案系統（VFS）
   ↓
Character/BLock 裝置層
   ↓
特定裝置驅動
   ↓
硬體
```

## UART（序列埠驅動）

UART（Universal Asynchronous Receiver-Transmitter）是 xv8 的主控台介面。程式碼在 `kernel/src/uart.rs`：

### UART 硬體程式設計

UART 使用記憶體映射 I/O，位址在 `0x88000000`（QEMU virt 機器）：

```rust
const UART_BASE: usize = 0x88000000;
// 暫存器偏移
const RHR: usize = 0;  // 接收保持暫存器（讀）
const THR: usize = 0;  // 傳輸保持暫存器（寫）
const IER: usize = 1;  // 中斷啟用暫存器
const FCR: usize = 2;  // FIFO 控制暫存器
const LCR: usize = 3;  // 線路控制暫存器
const LSR: usize = 5;  // 線路狀態暫存器
```

### 初始化

```rust
pub fn init() {
    // 停用中斷
    outb(UART_BASE + IER, 0);
    // 設定 LCR 為 8N1（8 位元、無同位、一個停止位）
    outb(UART_BASE + LCR, 3);
    // 啟用 FIFO
    outb(UART_BASE + FCR, 1);
    // 啟用接收中斷
    outb(UART_BASE + IER, 1);
}
```

### 讀寫操作

- 讀取：檢查 LSR 的 ready 位，從 RHR 讀取
- 寫入：等待 LSR 的空位，將資料寫入 THR

### console 整合

`console.rs` 包裝了 UART，提供更高層的介面：

```rust
pub fn consolewrite(buf: &[u8]) -> usize {
    for c in buf {
        uart::putc(*c);
    }
    buf.len()
}
```

## VirtIO 磁碟驅動

VirtIO 是一個虛擬化 I/O 框架，允許客戶作業系統訪問虛擬化資源。程式碼在 `kernel/src/virtio_disk.rs`。

### VirtIO 基礎

VirtIO 裝置使用 PCI 匯流排（由 `pci.rs` 列舉）。QEMU 的 virt 機器提供 virtio-blk 磁碟。

### VirtIO 描述符

VirtIO 使用描述符鏈（descriptor chains）來交換資料：

```rust
pub struct VirtqDesc {
    pub addr: u64,        // 緩衝區實體位址
    pub len: u32,         // 緩衝區長度
    pub flags: u16,        // VRING_DESC_F_NEXT 等
    pub next: u16,        // 下一個描述符索引
}
```

### 讀寫請求

磁碟請求通過 virtqueue 發送：

1. 準備請求描述符鏈（標頭、資料區域、回應區域）
2. 將描述符添加到 virtqueue 的可用環
3. 通知前端（VirtIO）有新的請求
4. 等待完成中斷
5. 從 virtqueue 的已完成環取出回應

### 緩衝區管理

`buf.rs` 的緩衝區快取與 VirtIO 整合：

- 讀取時，分配緩衝區並直接 DMA 到緩衝區
- 寫入時，將緩衝區標記為 dirty 並排程 I/O

## E1000 網卡驅動

Intel E1000（82540EM）是 QEMU 模擬的 PCIe 網卡。程式碼在 `kernel/src/e1000.rs`。

### PCI 發現

E1000 的 Vendor ID 為 0x8086，Device ID 為 0x100E：

```rust
const E1000_VENDOR_ID: u16 = 0x8086;
const E1000_DEVICE_ID: u16 = 0x100E;
```

PCI 枚舉找到 E1000 後：
1. 讀取 BAR0（Base Address Register）取得 MMIO 位置
2. 啟用 PCI 記憶體空間存取
3. 設定 DMA 相關暫存器

### MMIO 暫存器

E1000 使用記憶體映射 I/O，暫存器包括：

- `CTRL`：控制暫存器
- `STATUS`：狀態暫存器
- `EERD`/`EEWR`：EEPROM 讀寫（可選）
- `TDBAL`/`TDBAH`：發送描述符基址
- `RDBAL`/`RDBAH`：接收描述符基址
- `TDH`/`RDH`：描述符頭指標
- `TDT`/`RDT`：描述符尾指標

### 描述符環

E1000 使用 DMA 描述符環：

```rust
pub struct TxDesc {
    pub addr: u64,        // 緩衝區位址
    pub len: u32,         // 長度
    pub cmd: u8,          // 命令（EOP、RS 等）
    pub status: u8,        // 狀態（DD 表示完成）
    pub css: u8,           // 校驗和偏移
    pub special: u8,
}
```

### 發送流程

1. 分配一個 mbuf（DMA 緩衝區）
2. 複製資料到 mbuf
3. 準備 TxDescriptor
4. 將描述符寫入 TDAT 位置
5. 更新 TDT（尾指標）
6. 等待 TDTH 表示傳輸完成

### 接收流程

1. 準備 RxDescriptors 並更新 RDT
2. 等待接收中斷
3. 從 RDH 讀取完成的描述符
4. 處理接收的資料
5. 回收描述符並重新使用

## PLIC（平台層級中斷控制器）

PLIC（`kernel/src/plic.rs`）管理外部中斷的優先級和路由：

```rust
pub fn plic_init() {
    // 設定每個來源的優先級
    for i in 1..plic::PLIC_SIZE {
        *(PLIC_PRIORITY + i * 4) = 1;
    }
    // 啟用 UART 和 VirtIO 的中斷
    *(PLIC_HART0_SCLAIM + 0) = ...;
}
```

PLIC 的中斷處理：
1. 讀取 `CLAIM` 暫存器取得中斷源
2. 分派到對應的驅動程式處理
3. 寫入 `CLAIM` 確認中斷已處理

## 中斷處理流程

```rust
pub fn devintr() -> bool {
    let scause = scause::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            let irq = plic_claim();
            match irq {
                UART_IRQ => uart::intr(),
                VIRQ_IRQ => virtio_disk::intr(),
                E1000_IRQ => e1000::intr(),
                _ => {}
            }
            plic_complete(irq);
            true
        }
        _ => false
    }
}
```

## 計時器中斷

RISC-V 的計時器中斷在 `kernel/src/proc.rs` 中處理：

1. 設定下一次計時器中斷時間（`set_timer`）
2. 時間到時產生軟體中斷
3. 在 trap 處理中增加 `yield` 邏輯

## 相關主題

- [[RISC-V]]：PLIC 和中斷架構
- [[Trap]]：中斷如何被處理
- [[Network-Stack]]：E1000 如何與網路堆疊整合
- [[File-System]]：VirtIO 如何與檔案系統整合