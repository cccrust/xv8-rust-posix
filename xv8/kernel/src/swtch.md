# 上下文切換 — swtch.rs

swtch 函數實現兩個執行上下文之間的暫存器保存和恢復。

## 上下文結構

```rust
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Context {
    pub ra: usize,   // 返回位址
    pub sp: usize,   // 堆疊指標

    // callee-saved 暫存器
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}
```

## 為什麼只保存這些暫存器？

RISC-V 调用约定：
- **Caller-saved** (t0-t6, a0-a7)：由呼叫者保存，不需要在上下文切換時保存
- **Callee-saved** (s0-s11)：由被呼叫者保存，上下文切換需要保存

`ra`（返回位址）也很重要，因為 swtch 是透過 `ret` 返回的。

## swtch 實現

```rust
#[unsafe(naked)]
pub unsafe extern "C" fn swtch(old: &mut Context, new: &Context) {
    naked_asm!(
        // 保存當前暫存器到 old
        "sd ra, 0(a0)",
        "sd sp, 8(a0)",
        "sd s0, 16(a0)",
        "sd s1, 24(a0)",
        // ... 保存所有 s0-s11
        "sd s11, 104(a0)",

        // 從 new 恢復暫存器
        "ld ra, 0(a1)",
        "ld sp, 8(a1)",
        "ld s0, 16(a1)",
        // ... 恢復所有 s0-s11
        "ld s11, 104(a1)",

        "ret"
    );
}
```

## 使用場景

### 程序切換到排程器

```rust
// scheduler() 中
unsafe { swtch(&mut cpu.context, &proc.data().context) };
```

### 睡眠時切換到排程器

```rust
pub fn sched<'a>(proc_inner: SpinLockGuard<'a, ProcInner>, context: &mut Context) {
    let cpu = unsafe { current_cpu() };
    // ...

    let interrupts_enabled = cpu.interrupts_enabled;
    unsafe { swtch(context, &cpu.context) };
    // 恢復時從下一行繼續
}
```

## 堆疊使用

```
proc.context       cpu.context
┌──────────┐      ┌──────────┐
│    ra    │      │    ra    │  ← scheduler()
│    sp    │      │    sp    │
│    s0    │      │    s0    │
│   ...    │      │   ...    │
│   s11    │      │   s11    │
└──────────┘      └──────────┘
      │                  │
      ▼                  ▼
舊程序的核心堆疊  排程器的核心堆疊
```

## 安全性要求

```rust
// # Safety
// - 中斷必須停用
// - 不能在持有自旋鎖時呼叫（可能導致死鎖）
pub unsafe fn swtch(old: &mut Context, new: &Context)
```

## Naked 函數

`#[unsafe(naked)]` 表示：
- 函數體完全由內嵌組語組成
- 不生成 prologue/epilogue
- caller 不會保存/恢復任何暫存器

## 與 setjmp/longjmp 的比較

swtch 是手動實現的協作式多工：
- 需要明確呼叫 swtch
- 堆疊由作業系統控制
- 適合 Rust 的 zero-cost abstraction

## 相關主題

- [[proc]]：程序管理與排程
- [[trap]]：陷阱處理
- [[spinlock]]：鎖定與上下文切換