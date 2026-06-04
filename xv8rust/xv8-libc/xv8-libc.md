# xv8-libc

xv8 的核心 C 標準庫實作，提供最基礎的 C 函式庫功能。

## 設計

```toml
[package]
name = "xv8-libc"
version = "0.1.0"
edition = "2021"

[features]
default = []
std = []
```

無任何外部依賴，專為嵌入式/RISC-V 設計。

## 核心元件

### 字串處理

```rust
pub fn strlen(s: *const c_char) -> usize { ... }
pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char { ... }
pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void { ... }
```

### 記憶體操作

```rust
pub fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> i32 { ... }
pub fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void { ... }
```

### 格式化和輸出

```rust
pub fn printf(fmt: *const c_char, ...) -> i32 { ... }
pub fn sprintf(s: *mut c_char, fmt: *const c_char, ...) -> i32 { ... }
pub fn snprintf(s: *mut c_char, size: usize, fmt: *const c_char, ...) -> i32 { ... }
```

## 與 xv8-libc-compat 的關係

`xv8-libc-compat` 在 RISC-V 目標上調用 `xv8-libc`，兩者構成完整的 C 標準庫堆疊。

## Feature 標誌

- `std`：啟用需要作業系統支援的功能（如檔案 I/O）
- 無 features（default）：純核心功能

## 底層機制

xv8-libc 直接與硬體或 kernel 介面，不依賴作業系統。

## 使用場景

```rust
use xv8_libc::{printf, strlen};

unsafe {
    let s = b"Hello\0".as_ptr() as *const c_char;
    printf(b"Length: %d\n\0".as_ptr(), strlen(s));
}
```

## 與標準 libc 的差異

| 元件 | glibc/musl | xv8-libc |
|------|------------|----------|
| 依賴 | 複雜 | 無 |
| 功能 | 完整 | 精簡 |
| 目標 | 通用系統 | 嵌入式/OS |

## 程式碼位置

```
xv8rust/xv8-libc/src/
├── lib.rs       # 入口
├── string.rs    # 字串函式
├── stdio.rs     # 標準 I/O
└── alloc.rs     # 記憶體配置（可選）
```

## 穩定性

xv8-libc 專為 xv8 作業系統設計，API 穩定。

## 相關套件

- `xv8-libc-compat`：使用 xv8-libc 的包裝層
- `xv8-user-std`：依賴 xv8-libc 提供 std