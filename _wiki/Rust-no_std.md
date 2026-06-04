# Rust-no_std（無標準庫 Rust）

`no_std` Rust 指的是不連結標準庫 `std` 的 Rust 程式。這種模式常用於嵌入式系統、作業系統核心、以及需要完全控制環境的場景。

## 標準庫的組成

Rust 的 `std` 包含多個層次：

- `core`：無依賴的基本類型、迭代器、traits（可在任何環境使用）
- `alloc`：需要動態記憶體分配的功能（String、Vec、Box 等）
- `std`：完整的標準庫，依賴作業系統（檔案、網路、執行緒等）

`no_std` 程式碼通常使用 `core`（隱式可用）和 `alloc`（需要明確宣告）。

## xv8 的 no_std 使用

xv8 核心完全在 `no_std` 環境下開發：

```rust
// kernel/src/lib.rs
#![no_std]

extern crate alloc;

#[macro_use]
pub(crate) mod printf;
pub(crate) mod buf;
pub(crate) mod proc;
// ...
```

為什麼 xv8 核心需要 `no_std`：
1. 核心在作業系統「之下」，沒有作業系統服務可用
2. 需要完全控制記憶體配置和例外處理
3. 不能依賴可能使用作業系統功能的程式庫

## 恐慌處理

`std` 程式遇到 panic 時會呼叫 `std::panic::panic_hook`。`no_std` 程式沒有這個，需要自訂：

```rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    if let Some(loc) = info.location() {
        print!("panicked at {}:{}:{}",
            loc.file(), loc.line(), loc.column());
    }
    loop {}
}
```

`panic_handler` 屬性告訴編譯器哪個函式處理 panic。

## 記憶體分配

`no_std` 環境沒有預設的堆積。xv8 使用簡單的物件池：

```rust
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
```

`alloc` 依賴全局記憶體分配器。需要提供 `#[global_allocator]`：

```rust
use core::alloc::{GlobalAlloc, Layout};

struct SimpleAllocator;

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 從預先分配的記憶體池分配
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // 返回到記憶體池
    }
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator;
```

xv8 的核心記憶體分配器（`kalloc.rs`）使用伙伴系統。

## 符號別名與符號注入

`no_std` 程式的進入點不是 `main`，而是 `lang_start`（由 `std` 提供）。`no_std` 程式需要自己處理初始化。

xv8 的 `kernel/src/entry.rs` 定義了進入點：

```rust
#[no_mangle]
extern "C" fn _start() {
    xv8::main()
}
```

`#[no_mangle]` 屬性防止編譯器修改符號名稱，確保連結器能找到它。

## 內嵌組語

與硬體互動需要內嵌組語（inline assembly）。RISC-V 的 `ecall` 指令：

```rust
unsafe {
    core::arch::asm!(
        "ecall",
        in("a7") syscall_num,
        in("a0") arg0,
        lateout("a0") ret
    );
}
```

`asm!` 巨集允許在 Rust 程式碼中嵌入目標架構的組語。

## 屬性語法

`no_std` 程式使用屬性來啟用功能和解釋器選項：

```rust
#![no_std]                    // 不連結 std
#![feature(asm)]              // 啟用內嵌組語（nightly）
#![feature(lang_items)]       // 需要自訂 lang items
#![feature(allocator_api)]    // 使用分配器 API
```

## 與 xv8-std 的關係

xv8-std 是 xv8 對 `std` 的「覆寫」。它提供：
- `File`、`Read`、`Write` 等高層抽象
- `println!`、`format!` 等巨集
- `Vec`、`String` 等分配類型

但底層仍然是 `no_std` + `alloc`，沒有作業系統依賴。

## 何時使用 no_std

適合使用 `no_std` 的場景：
1. **作業系統核心**：沒有作業系統服務可用
2. **嵌入式系統**：記憶體和資源受限
3. **bootloader**：在作業系統載入之前執行
4. **效能關鍵路徑**：不想承擔 std 的啟動成本

## 與 std 的對比

| 方面 | std | no_std |
|------|-----|--------|
| 記憶體分配 | 總是可用 | 需要 `alloc` + 自訂分配器 |
| I/O | 檔案、網路等 | 需自己實現 |
| 執行緒 | `std::thread` | 需自己實現或使用 `core` |
| 恐慌處理 | 預設實現 | 需自訂 `panic_handler` |
| 進入點 | `fn main()` | `#[no_mangle] extern "C" fn _start()` |

## 過渡到 std

有時需要從 `no_std` 切換到 `std`。在 xv8 的 `user/` 子系統中：

```rust
// user/src/lib.rs
#![no_std]
extern crate alloc;
```

使用者空間程式使用 `alloc` 但仍然沒有完整的 `std`。要使用完整的 `std`（如在真正的應用程式中），需要：

```rust
fn main() {
    println!("Hello, world!");
}
```

這會連結完整的 `std`。

## 相關主題

- [[xv8-std]]：std 覆寫層
- [[RISC-V]]：RISC-V 內嵌組語
- [[Cross-Compilation]]：no_std 環境的交叉編譯
- [[Device-Drivers]]：如何使用內嵌組語與硬體互動