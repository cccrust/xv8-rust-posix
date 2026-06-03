# crossterm (vendored)

跨平台終端操作庫，已 vendored 到 xv8 專案。

## 設計

```toml
[package]
name = "crossterm"
version = "0.29.0"
```

此版本從 crates.io 複製而來，專為 xv8 的 vi/vim 編輯器使用。

## 原始來源的差異

原始 crossterm 是純 Rust 的終端操作庫，xv8 的版本經過修改：
- 移除了在 RISC-V 上不需要的功能
- 保留了核心的螢幕操作和 ANSI 轉義序列

## 主要功能

### 螢幕操作

```rust
use crossterm::{ ExecutableCommand, terminal };
terminal::enable_raw_mode().unwrap();
stdout().execute(terminal::Clear(ClearType::All)).unwrap();
```

### 游標控制

```rust
use crossterm::cursor;
stdout().execute(cursor::MoveTo(x, y)).unwrap();
```

### 顏色輸出

```rust
use crossterm::style;
println!("{}", style("Red").red());
```

## xv8 中的使用

vi/vim 編輯器需要 crossterm 提供的功能：

```toml
[features]
default = ["crossterm"]
crossterm = ["dep:crossterm"]

[[bin]]
name = "vi"
required-features = ["crossterm"]
```

## 與原始 crossterm 的相容性

xv8 的 crossterm 盡可能相容原始 API，但：
- 移除了信號處理（不需要）
- 移除了事件輪詢（終端模式不同）
- 簡化了終端檢測

## 底層原理

Crossterm 生成 ANSI 轉義序列：
- `\x1b[2J`：清除螢幕
- `\x1b[H`：移動到頂端
- `\x1b[31m`：紅色文字

現代終端支援這些序列，但純文字模式（如 xv8）也可能支援。

## 依賴

```toml
[dependencies]
bitflags = "2.9"
spin = "0.9"

[target.'cfg(not(target_arch = "riscv64"))'.dependencies]
parking_lot = "0.12"
```

- `bitflags`：標誌位元組處理
- `spin`：自旋鎖（Unix 目標）

## RISC-V 考量

```toml
[target.'cfg(target_arch = "riscv64")'.dependencies]
xv8-libc = { path = "../xv8-libc" }
std = { package = "xv8-user-std", path = "../xv8-user-std" }
```

RISC-V 目標使用 xv8 的 libc 和 std。

## 為何 Vendored

1. 離線可用
2. 版本固定
3. 無需網路下載
4. 可針對 xv8 修改

## 限制

xv8 環境中的終端功能有限，某些 ANSI 功能可能無法使用。

## 相關套件

- `xv8-user-std`：終端輸出的底層
- `xv8-libc`：字元 I/O