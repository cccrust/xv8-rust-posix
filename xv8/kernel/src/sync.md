# 同步原語 — sync.rs

## 概述

`sync.rs` 提供 xv8 核心使用的同步原語，包裝標準的 `spin::Mutex` 與 `once_cell::sync::OnceCell`，提供核心專屬的 `OnceLock`。

## 實作

### OnceLock

```rust
pub struct OnceLock<T> {
    inner: OnceCell<T>,
}
```

`OnceLock` 是 xv8 核心的全域初始化原語，基於 `once_cell::sync::OnceCell`。用於一次性的核心資料結構初始化：

```rust
static GLOBAL_STATE: OnceLock<SomeStruct> = OnceLock::new();
GLOBAL_STATE.get_or_init(|| SomeStruct::new());
```

### 使用場景

- **核心全域變數**: 在 `main()` 初始化前不能存取的資料結構
- **裝置驅動程式**: 在 probe 階段初始化的裝置狀態
- **檔案系統**: 在 mount 階段初始化的超級區塊

## 相關文件

- [spinlock 文件](spinlock.md)
- [sleeplock 文件](sleeplock.md)
