# QEMU（快速模擬器）

QEMU 是一個通用的開放原始碼機器模擬器。在 xv8 開發中，QEMU 用於在沒有真實 RISC-V 硬體的情況下執行和測試 xv8 作業系統。

## 為何使用 QEMU

開發 xv8 需要：
- 能在常見的主機（x86_64/aarch64 macOS 或 Linux）上執行
- 模擬 RISC-V 架構
- 支援周邊裝置（序列埠、網路、磁碟）
- 提供良好的除錯支援

QEMU 滿足所有這些需求，而且是自由軟體。

## 安裝 QEMU

### macOS

```bash
brew install qemu
# 或使用 MacPorts
sudo port install qemu
```

### Linux (Ubuntu/Debian)

```bash
sudo apt install qemu-system-riscv64
```

### 驗證安裝

```bash
qemu-system-riscv64 --version
# 輸出：QEMU emulator version 8.x.x
```

## QEMU 執行 xv8

### 基本命令列

```bash
qemu-system-riscv64 \
    -machine virt \           # virt 機器類型（常用於 RISC-V）
    -cpu max \                # 使用最大配置的 CPU
    -m 256M \                 # 256 MB 記憶體
    -kernel kernel.elf \      # 核心 ELF 檔
    -append "rootfstype=ext4" \  # 核心參數
    -drive file=fs.img,if=virtio \  # VirtIO 磁碟
    -serial mon:stdio \       # 序列輸出到 stdout
    -nographic                # 無圖形介面
```

### 使用 xv8 的 cargo run

xv8 的 `xv8/.cargo/config.toml` 已經配置好了：

```toml
[run]
target = "riscv64gc-unknown-none-elf"
```

直接執行：

```bash
cargo run --release
```

會自動呼叫 QEMU。

## QEMU 機器類型

QEMU 支援多種 RISC-V 機器：

| 機器 | 描述 |
|------|------|
| virt | 通用虛擬機器，支援大多數 VirtIO 裝置 |
| sifive_u | SiFive HiFive 開發板模擬 |
| spike | RISC-V 參考實現（spike ISA 模擬器） |

`virt` 是最常用的，專為虛擬化環境優化。

## 模擬的周邊裝置

### UART（序列埠）

QEMU 的 virt 機器模擬了一個 16550 相容的 UART，映射到 `0x88000000`。這對應 xv8 的 UART 驅動程式。

### VirtIO

VirtIO 是一個高效的虛擬化周邊介面。QEMU 的 virt 機器支援：

- **VirtIO Block**（磁碟）：`if=virtio` 或 `-device virtio-blk-device`
- **VirtIO Network**（網卡）：`-netdev` + `-device virtio-net-device`

xv8 使用 VirtIO 磁碟來獲得高效能的磁碟 I/O。

### E1000 網卡

除了 VirtIO，QEMU 還模擬 Intel E1000 網卡：

```
-device e1000,netdev=net0
```

xv8 有專門的 E1000 驅動程式（`e1000.rs`）。這與 xv6 不同，xv6 使用 VirtIO 網卡。

### PLIC（中斷控制器）

Platform-Level Interrupt Controller (PLIC) 模擬多個中斷源：
- UART（韌體控制台輸入）
- VirtIO 磁碟
- E1000 網卡

xv8 的 `plic.rs` 處理這些中斷。

## 網路設定

### 使用者模式 NAT

```bash
-netdev user,id=net0,hostfwd=tcp::5555-:80
-device e1000,netdev=net0
```

這設定使用者模式網路：
- xv8 獲得 IP（通常 10.0.2.15）
- 主機的 5555 連接埠轉發到 xv8 的 80 連接埠

### TAP 模式

對於更先進的網路（需要 xv8 能被主機直接訪問）：

```bash
-tap network=shared
```

需要 root 權限和正確的網路設定。

## QEMU 與 GDB 除錯

QEMU 支援通過 GDB 遠端除錯：

```bash
qemu-system-riscv64 \
    -machine virt \
    -s -S \       # -s: 啟動 GDB 伺服器（:1234），-S: 啟動時暫停
    -kernel kernel.elf
```

然後在另一個終端：

```bash
riscv64-unknown-elf-gdb kernel.elf
(gdb) target remote localhost:1234
(gdb) break _start
(gdb) continue
```

`-s` 是 `-gdb tcp::1234` 的簡寫。

## QEMU 的檔案系統映射

xv8 使用一個磁碟映像檔 (`fs.img`)：

```bash
-hda fs.img
# 或
-drive file=fs.img,if=virtio
```

mkfs 工具（在 `mkfs/` 目錄）用於建立這個映像檔。

## 常見問題

### QEMU 啟動但沒有輸出

檢查：
1. 核心是否正確編譯
2. 使用的核心 ELF 是否是 RISC-V 架構：`file kernel.elf`
3. 序列埠是否正確映射

### 網路無法連線

1. 確認 xv8 的網路堆疊正確初始化
2. 檢查 QEMU 的 `-netdev` 設定
3. 嘗試在 xv8 中 ping 10.0.2.2（閘道）

### 效能問題

QEMU 在模擬模式下（軟體模擬）比較慢。可以：
- 使用更多 CPU 核心（`-smp 4`）
- 增加記憶體（`-m 512M`）
- 使用 KVM 加速（如果在 Linux 上且支援）

## QEMU 與 xv8 測試

xv8 的 `./test.sh` 腳本會：

1. 編譯核心和使用者程式
2. 建立檔案系統映像
3. 在 QEMU 中執行 xv8
4. 執行內部測試（testrunner）

## 替代方案

除了 QEMU，還有其他 RISC-V 模擬選擇：

- **spike**：RISC-V 參考 ISA 模擬器（較慢，但更嚴格）
- **renode**：用於嵌入式系統的模擬器
- **真實硬體**：SiFive HiFive 或類似的開發板

QEMU 仍然是最實用且廣泛支援的選擇。

## 相關主題

- [[RISC-V]]：QEMU 模擬的 RISC-V 架構
- [[Device-Drivers]]：QEMU 模擬的 UART、VirtIO、E1000
- [[Network-Stack]]：QEMU 提供的網路功能
- [[Cross-Compilation]]：如何在主機上交叉編譯核心