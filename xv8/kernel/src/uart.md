# 序列埠 — uart.rs

UART（Universal Asynchronous Receiver-Transmitter）是用於 QEMU 與主機通訊的序列埠驅動。

## QEMU 虛擬 UART

```
QEMU 主機                      xv8 客戶機
    │                              │
    │◄─────── UART0 ─────────────►│
    │     (0x1000_0000)            │
    │                              │
    │      16550 相容              │
    │                              │
    └──────────────────────────────┘
              │
              │ stdio
              ▼
        主機終端
```

## 寄存器映射

```rust
const RHR: usize = 0;   // Receive Holding Register (input)
const THR: usize = 0;   // Transmit Holding Register (output)
const IER: usize = 1;   // Interrupt Enable Register
const FCR: usize = 2;   // FIFO Control Register
const ISR: usize = 2;   // Interrupt Status Register
const LCR: usize = 3;   // Line Control Register
const LSR: usize = 5;   // Line Status Register
```

## 狀態標誌

```rust
const LSR_RX_READY: u8 = 1 << 0;  // 可讀取
const LSR_TX_IDLE: u8 = 1 << 5;  // 傳送完成
```

## 初始化

```rust
pub fn init(&mut self) {
    // 停用中斷
    self.write_reg(IER, 0x00);

    // 設定鮑率：38.4K
    self.write_reg(LCR, LCR_BAUD_LATCH);  // 特殊模式
    self.write_reg(0, 0x03);               // LSB
    self.write_reg(1, 0x00);              // MSB
    self.write_reg(LCR, LCR_EIGHT_BITS);  // 8N1

    // 重設並啟用 FIFO
    self.write_reg(FCR, FCR_FIFO_ENABLE | FCR_FIFO_CLEAR);

    // 啟用傳送和接收中斷
    self.write_reg(IER, IER_TX_ENABLE | IER_RX_ENABLE);
}
```

## 讀取字元

```rust
pub fn getc() -> Option<u8> {
    // 不需要鎖，因為只讀取接收側
    let uart = unsafe { UART.get_mut_unchecked() };

    if uart.read_reg(LSR) & LSR_RX_READY != 0 {
        Some(uart.read_reg(RHR))
    } else {
        None
    }
}
```

## 中斷驅動寫入

```rust
pub fn write(buf: &[u8]) {
    let mut uart = UART.lock();

    for c in buf {
        // 等待 UART 空閒
        while uart.tx_busy {
            uart = proc::sleep(
                Channel::Buffer(&uart.tx_channel as *const _ as usize),
                uart
            );
        }

        // 傳送字元
        uart.write_reg(THR, *c);
        uart.tx_busy = true;
    }
}
```

## 同步寫入

```rust
pub fn write_sync(buf: &[u8]) {
    // 如果某核心已恐慌，停止其他核心
    if PRINTF.is_panicked() {
        loop { core::hint::spin_loop() }
    }

    // 如果已持有鎖，跳過取得（避免恐慌時死鎖）
    let _guard = (!UART.is_holding()).then(|| UART.lock());

    let uart = unsafe { UART.get_mut_unchecked() };

    for c in buf {
        // 輪詢直到傳送完成
        while (uart.read_reg(LSR) & LSR_TX_IDLE) == 0 {}
        uart.write_reg(THR, *c);
    }
}
```

## 中斷處理

```rust
pub fn handle_interrupt() {
    {
        let mut uart = UART.lock();

        // 確認中斷
        uart.read_reg(ISR);

        if (uart.read_reg(LSR) & LSR_TX_IDLE) != 0 {
            // 傳送完成，喚醒等待者
            uart.tx_busy = false;
            proc::wakeup(Channel::Buffer(&uart.tx_channel as *const _ as usize));
        }
    }

    // 讀取並處理輸入字元
    while let Some(c) = getc() {
        Console::handle_interrupt(c);
    }
}
```

## 寄存器讀寫

```rust
fn read_reg(&self, reg: usize) -> u8 {
    unsafe { ptr::read_volatile((self.base_address as *mut u8).add(reg)) }
}

fn write_reg(&mut self, reg: usize, value: u8) {
    unsafe { ptr::write_volatile((self.base_address as *mut u8).add(reg), value) }
}
```

`volatile` 确保每次都产生实际内存访问。

## 與 Console 的整合

```
使用者程式 read(STDIN)
        ↓
console.rs::read()
        ↓
UART 中斷觸發
        ↓
uart.rs::handle_interrupt()
        ↓
console.rs::handle_interrupt()  // 處理輸入
        ↓
喚醒 sleeping read()
```

## 波特率設定

```rust
// 38.4K 波特率
// 除數 = 115200 / 38400 = 3
write_reg(0, 3);   // LSB
write_reg(1, 0);   // MSB
```

## 相關主題

- [[console]]：主控台處理
- [[trap]]：中斷處理
- [[memlayout]]：記憶體映射