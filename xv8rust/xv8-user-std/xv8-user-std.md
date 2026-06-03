# xv8-user-std

xv8 的 user-space std overlay，讓原本需要標準庫的 Rust 程式可以在 xv8 上運行。

## 設計

```toml
[package]
name = "xv8-user-std"
version = "0.1.0"
edition = "2021"

[dependencies]
xv8-libc = { path = "../xv8-libc" }
hashbrown = "0.15"
```

## 核心概念

### 繞過標準庫限制

```rust
#![no_std]
#![no_main]
```

在 xv8 上運行的程式碼不能使用標準 Rust 庫，需要這個 overlay。

## 實作的 traits

### Write

```rust
impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        xv8_libc::write(1, buf.as_ptr(), buf.len());
        Ok(buf.len())
    }
}
```

### Read

```rust
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = xv8_libc::read(0, buf.as_mut_ptr(), buf.len());
        Ok(n as usize)
    }
}
```

## HashMap 支援

```rust
hashbrown = "0.15"
```

由於標準庫的 HashMap 依賴allocator，hashbrown 提供了獨立的實現。

## alloc

```rust
#[global_allocator]
static ALLOC: SomeAllocator = SomeAllocator::new();
```

提供全域記憶體配置器，掛鉤到 xv8 的記憶體系統。

## 與普通 std 的差異

| 元件 | 標準 std | xv8-user-std |
|------|----------|--------------|
| 輸出 | stdout | xv8 console |
| 輸入 | stdin | xv8 console |
| 檔案 | VFS | xv8 fs |
| 網路 | kernel socket | xv8 net stack |

## 使用方式

POSIX 工具在 RISC-V 目標時：
```toml
[target.'cfg(target_arch = "riscv64")'.dependencies]
std = { package = "xv8-user-std", path = "../../xv8rust/xv8-user-std" }
```

## 底層呼叫鏈

```
Rust code (println!)
    → xv8-user-std (Write trait)
        → xv8-libc::write()
            → syscall
                → xv8 kernel
```

## 限制

xv8-user-std 實現了核心功能，但並非完整的 std：
- 尚無完整檔案系統 API
- 網路支援有限
- 執行緒未實現

## 相關套件

- `xv8-libc`：底層 C 庫
- `xv8-libc-compat`：syscall 包裝