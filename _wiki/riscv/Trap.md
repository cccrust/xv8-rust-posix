# Trap

Trap 是 RISC-V 用於處理例外的機制，讓使用者程式可以與核心互動。在 xv8 中，trap 包括系統呼叫、頁錯誤、裝置中斷等。

## Trap 的種類

RISC-V 區分兩類異常：

1. **Exception（例外）**：由指令執行同步引發
   - 系統呼叫（ecall）
   - 非法指令
   - 頁錯誤
   - 存取錯誤

2. **Interrupt（中斷）**：由硬體非同步觸發
   - 計時器中斷（時鐘）
   - 軟體中斷（核間通訊）
   - 外部中斷（UART、VirtIO、E1000）

`scause` CSR 描述 trap 的原因。

## Trap 向量

RISC-V 使用 `stvec` CSR 設定 trap 向量位址。xv8 的設計：

- 所有同步例外（ecall、頁錯誤等）都跳到同一個向量
- `kernelvec`（`kernel/src/kernelvec.rs`）在核心態處理

`stvec` 可以設定為：
- **直接模式**：所有 trap 跳到同一個位址
- **向量模式**：不同類型的 trap 跳到不同偏移

xv8 使用直接模式，所有 trap 都到 `kernelvec`。

## Trap 進入流程

當使用者程式執行 `ecall` 或發生錯誤時，硬體自動：

1. 將 `sstatus` 儲存到 `sscratch`（或相反，根據模式）
2. 儲存 `sepc`（發生例外的 PC）
3. 儲存 `scause`（例外原因）
4. 將程式計數器設為 `stvec`
5. 切換到監督者模式

xv8 的 `kernelvec` 處理：

```rust
kernelvec:
    # 保存使用者暫存器到 trapframe
    sd ra, 0(a0)
    sd sp, 8(a0)
    # ... 保存其他通用暫存器
    # 切換到核心堆疊
    # 呼叫 trap.rs 的處理函式
```

## Trapframe 結構

每個程序有一個 trapframe（`user/src/syscall.rs`）用於儲存使用者暫存器：

```rust
pub struct Trapframe {
    pub kernel_satp: usize,    // 核心的 satp
    pub kernel_sp: usize,      // 核心堆疊指標
    pub kernel_trap: usize,    // trap 處理後返回的位址
    pub epc: usize,             // 使用者程式的 EPC（exception PC）
    pub kernel_hartid: usize,   // HART ID
    pub ra: usize, pub sp: usize, pub gp: usize,
    pub tp: usize, pub t0-t6: [usize; 7],
    pub a0-a7: [usize; 8],
    // ... 更多暫存器
}
```

## Trap 處理解析

`trap.rs` 中的 `usertrap()` 是主要的 trap 處理函式：

```rust
pub fn usertrap() {
    let cause = scause::read().cause();
    match cause {
        Trap::Exception(Exception::UserEcall) => syscall(),
        Trap::Exception(Exception::LoadPageFault) => handle_page_fault(),
        Trap::Exception(Exception::StorePageFault) => handle_page_fault(),
        Trap::Interrupt(Interrupt::SupervisorTimer) => yield_cpu(),
        Trap::Interrupt(Interrupt::SupervisorExternal) => devintr(),
        _ => panic!("unhandled trap"),
    }
}
```

## 系統呼叫處理

當 `scause` 指示使用者 ecall 時：

1. `usertrap` 呼叫 `syscall()`
2. `syscall()` 從 `a7` 取得系統呼叫號碼
3. 根據號碼分派到對應的處理函式（如 `sys_open`、`sys_read` 等）
4. 處理完成後，呼叫 `usertrapret()`

## 頁錯誤處理

當程式存取未映射的虛擬位址時，發生頁錯誤：

1. `scause` 包含錯誤類型（load store instruction）
2. `stval` 包含造成錯誤的虛擬位址
3. 核心檢查該位址是否在合法的記憶體區域內
4. 如果是，分配物理頁並建立映射（lazy allocation 或 COW）
5. 如果不是，發送 SIGSEGV 信號給程式或終止

## 中斷處理

### 計時器中斷

RISC-V 的計時器中斷由 `sstatus` 中的 `SIE` 位控制：

1. 時鐘中斷發生
2. 核心標記需要排程
3. 在 `usertrap` 或排程點檢查並呼叫 `yield`

### 外部中斷（PLIC）

外部中斷（UART、VirtIO、E1000）通過 PLIC（Platform Level Interrupt Controller）路由：

1. 裝置產生中斷
2. PLIC 分配優先級並路由到 CPU
3. `sip` 中的 SEIP 位被設定
4. `devintr()` 讀取 PLIC 來確定哪個裝置
5. 分派到該裝置的中斷處理常式

## Trap 返回

處理完成後，透過 `usertrapret()` 返回使用者模式：

```rust
pub fn usertrapret() {
    // 準備 uservec 的指標和參數
    w_satp(phys_to_virt(user_page_table) | (ASID << 44));
    // 設定 stvec 為 uservec
    w_stvec(USER_VEC as usize);
    // 返回使用者模式
    sret;
}
```

`sret` 指令從 `sepc` 恢復 PC 並返回使用者模式。

## 使用者態的 Trap 向量

`uservec`（在 `trampoline.S` 或 `user/src/trap.rs`）是使用者空間的 trampoline：

1. 將通用暫存器儲存到 trapframe
2. 切換到核心頁表
3. 跳到 `kernelvec`

這個 trampoline 是必要的，因為切換頁表前需要先保存狀態。

## 信號傳遞

當核心需要發送信號給使用者程式時：

1. 修改 trapframe 中的 `epc` 指向信號處理常式
2. 設定 `a0` 為信號編號
3. 設定 `a1` 為信號處理常式的指標（sigaction）
4. 返回使用者模式時會跳到信號處理常式

## 相關主題

- [[RISC-V]]：CSR 和特權模式
- [[Syscall]]：ecall 觸發的 trap
- [[Process]]：程序的 trapframe 管理
- [[Device-Drivers]]：PLIC 和外部中斷