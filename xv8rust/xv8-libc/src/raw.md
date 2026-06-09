# Raw — 原始系統呼叫包裝

`raw.rs` 包含 52 個系統呼叫的原始包裝，每個函式使用 Rust 的 `core::arch::asm!` 巨集嵌入 RISC-V `ecall` 指令。

## 系統呼叫機制

`ecall` 指令從使用者模式（U 模式）陷入監督者模式（S 模式），xv8 核心的 trap 處理器捕捉後根據 a7 暫存器的值分派：

```rust
fn sys_write(fd: usize, buf: *const u8, count: usize) -> isize {
    let ret;
    unsafe {
        asm!("ecall",
            in("a7") SYS_write,
            in("a0") fd,
            in("a1") buf,
            in("a2") count,
            lateout("a0") ret,
            options(nostack));
    }
    ret
}
```

## 回傳值處理

核心約定回傳值在 a0 暫存器。負數值表示錯誤（`-errno`）。xv8-libc 不處理此轉換，由呼叫者或上層 xv8-libc-compat 層進行轉換。

## 實作的系統呼叫範圍

52 個系統呼叫涵蓋：
- 行程控制（fork、exec、exit、wait、getpid、clone）
- 記憶體（sbrk、mmap、munmap、mprotect）
- 檔案系統（open、read、write、close、lseek、stat、mkdir、unlink）
- 目錄（chdir、getcwd、readdir）
- 網路（socket、bind、listen、accept、connect、send、recv）
- 時間（time、clock_gettime、nanosleep）
- 同步（futex、pause）
- 訊號（kill、sigaction、rt_sigprocmask、signalfd）
- 其他（getpid、getuid、gethostname、brk）

## 相關文件

- [args.md](./args.md) — 系統呼叫參數打包
- [lib.md](./lib.md) — xv8-libc 總覽
- [syscall.md](../../kernel/src/syscall.md) — 核心系統呼叫處理
