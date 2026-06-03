# Cross-Compilation（交叉編譯）

交叉編譯是在一個平台上構建可在另一個不同架構的平台上執行的程式。在 xv8-rust-posix 中，需要在 x86_64/apple-darwin 主機上編譯出能在 RISC-V 目標上執行的二進位檔。

## 為何需要交叉編譯

xv8 是一個 RISC-V 作業系統，只能在 RISC-V 硬體或模擬器（如 QEMU）上執行。但大多數開發者的電腦是 x86_64（Intel/AMD）或 aarch64（Apple Silicon）。交叉編譯讓我們能在本機構建，直接在 QEMU 中執行測試。

## Rust 交叉編譯基礎

### 目標三元組

Rust 使用目標三元組（target triple）來識別編譯目標：

```
架構 - 供應商 - 作業系統 - ABI
```

- **主機**：`x86_64-apple-darwin`（Intel Mac）或 `aarch64-apple-darwin`（Apple Silicon Mac）
- **目標**：`riscv64gc-unknown-none-elf`
  - `riscv64`：64 位元 RISC-V
  - `gc`：包含 G（通用）、C（壓縮）擴充
  - `unknown-none-elf`：無作業系統（bare metal），使用 ELF 格式

### 添加目標

```bash
rustup target add riscv64gc-unknown-none-elf
```

這會安裝 RISC-V 目標的標準庫。

## Cargo 配置

### 目標設定

在 `posix/.cargo/config.toml` 中設定交叉編譯目標：

```toml
[build]
target = "riscv64gc-unknown-none-elf"

[target.riscv64gc-unknown-none-elf]
linker = "riscv64-unknown-elf-gcc"
rustflags = ["-C", "link-arg=-Tuser.ld"]
```

### 連結器

`riscv64-unknown-elf-gcc` 是 RISC-V 工具鏈的 GCC，用於：
- 將物件檔案連結成 ELF 可執行檔
- 提供 `no_std` 環境所需的啟動檔（crt0）

在 macOS 上可能需要使用 homebrew 安裝的 `riscv64-elf-gcc` 或從別的路徑調用。

## Rustc Wrapper 機制

xv8 的巧妙設計：`root/.cargo/rustc-wrapper.sh`

```bash
#!/bin/bash
# 注入 no_main 和 no_mangle 屬性

if [ "$1" == "build" ] && echo "$@" | grep -q "riscv64gc-unknown-none-elf"; then
    # 為 riscv64 目標包裝編譯過程
    exec rustc "$@" \
        --edition 2021 \
        -C "llvm-args=--noop" \
        2>&1
else
    exec rustc "$@"
fi
```

這個 wrapper 在交叉編譯時注入必要屬性，解決了 `main` 函式符號問題。

## build-std 機制

xv8rust 和 posix 需要完整的核心庫才能編譯。在 `config.toml` 中：

```toml
[build]
target = "riscv64gc-unknown-none-elf"
rustflags = ["-C", "link-arg=-Tuser.ld"]

[profile.release]
lto = true
strip = true
codegen-units = 1
```

`build-std` 讓 Rust 編譯器自己編譯 `core` 和 `alloc`，而不是使用預編譯的版本。這對 `no_std` 程式至關重要。

## 編譯命令

### 編譯 xv8rust

```bash
cargo build --release --manifest-path xv8rust/Cargo.toml --target riscv64gc-unknown-none-elf
```

### 編譯 POSIX 工具

```bash
cargo build --release --manifest-path posix/Cargo.toml --target riscv64gc-unknown-none-elf --no-default-features
```

`--no-default-features` 排除主機特定依賴（如 `libc` 的主機特定部分）。

### 驗證編譯成功

```bash
file target/riscv64gc-unknown-none-elf/release/ls
# 輸出應包含：ELF 64-bit LSB executable, UCG RISC-V
```

## 執行環境：QEMU

編譯完成後，需要在 QEMU 中執行。

### QEMU 安裝

```bash
brew install qemu
# 或
apt install qemu-system-riscv64
```

### QEMU 執行

```bash
cd xv8
cargo run --release
# 或手動：
qemu-system-riscv64 \
    -machine virt \
    -cpu max \
    -m 256M \
    -kernel target/riscv64gc-unknown-none-elf/release/xv8 \
    -append "rootfstype=ext4" \
    -drive file=fs.img,if=virtio,discard=unmap \
    -netdev user,id=net0 \
    -device e1000,netdev=net0 \
    -bios default
```

### QEMU 網路設定

`-netdev user,id=net0` 提供使用者模式 NAT：
- xv8 獲得 IP（通常 10.0.2.15）
- 10.0.2.2 是閘道器
- 10.0.2.3 是 DNS 伺服器

## 常見問題

### 連結器錯誤

如果出現：
```
error: linker `riscv64-unknown-elf-gcc` not found
```

需要安裝 RISC-V 工具鏈：
```bash
brew install riscv-gnu-toolchain
```

### 缺少 target

如果 Rust 不認識目標：
```bash
rustup target add riscv64gc-unknown-none-elf
```

### no_std 恐慌

`no_std` 程式遇到 panic 時會直接停止。xv8rust 提供自訂 panic handler：
```rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    // 輸出錯誤資訊後進入無限循環
    loop {}
}
```

## 工具鏈版本

xv8 需要 nightly Rust：

```toml
# xv8/rust-toolchain.toml
[toolchain]
channel = "nightly"
```

nightly 版本提供 `no_std` 和內嵌組語等穩定前功能。

## 與 xv8-std 的整合

交叉編譯時，`xv8rust/Cargo.toml` 指定的依賴（如 `libc`、`alloc`）都需要有 `no_std` 版本。xv8-libc-compat 提供 `libc` 的 `no_std` 實作。

## 相關主題

- [[xv8-std]]：std 覆寫層
- [[libc-compat]]：C 標準庫相容層
- [[Rust-no_std]]：無標準庫 Rust 程式設計
- [[QEMU]]：RISC-V 模擬器