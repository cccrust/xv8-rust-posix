# Args — 系統呼叫參數打包

`args.rs` 定義了 xv8 系統呼叫的參數打包與解包工具。RISC-V 的系統呼叫約定使用暫存器 `a0`–`a5` 傳遞最多六個參數。`args.rs` 提供型別安全的封裝，將 Rust 的型別系統對映到這些暫存器。

## 參數傳遞規範

RISC-V Linux 系統呼叫 ABI 規範：

- **a7**: 系統呼叫編號
- **a0**: 第一個參數（也作為回傳值暫存器）
- **a1**: 第二個參數
- **a2**: 第三個參數
- **a3**: 第四個參數
- **a4**: 第五個參數
- **a5**: 第六個參數

## 封裝模式

`args.rs` 定義 `SyscallArgs` 結構，提供 `pack` 與 `unpack` 方法。指標參數（如 `*const u8`）需轉為 `usize` 傳遞。回傳的 `isize` 值若為負數，表示錯誤（對應 `-errno`）。

## 相關文件

- [abi.md](../../kernel/src/abi.md) — 核心 ABI 定義
- [raw.md](./raw.md) — 原始系統呼叫包裝
- [lib.md](./lib.md) — xv8-libc 總覽
