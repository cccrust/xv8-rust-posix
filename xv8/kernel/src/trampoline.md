# Trampoline — 使用者/核心模式切換

## 概述

`trampoline.rs` 包含使用者模式與核心模式之間切換的關鍵組合語言程式碼。trampoline 頁面（trampoline page）是唯一同時映射在使用者空間與核心空間的頁面，作為模式切換的橋樑。

## 雙重映射

trampoline 頁面位於虛擬位址空間的頂端：

```
0xFFFFFFFFFFFFF000 --> trampoline page (核心視角)
0xFFFFFFFFFFFFF000 --> trampoline page (使用者視角)
```

透過在所有頁表中（核心頁表與每個程序的頁表）在此位址映射同一實體頁面，確保切換頁表時指令流程不中斷。

## satp 切換

`sapt`（Supervisor Address Translation and Protection）CSR 控制分頁機制。從使用者切入核心時：

1. **uservec 執行**：當使用者程式觸發陷阱時，CPU 跳轉至 `stvec`（指向 trampoline 頁面上的 `uservec`）
2. **儲存暫存器**：將使用者暫存器存入對應的程序 trapframe
3. **替換 satp**：將 `satp` 從使用者頁表切換至核心頁表
4. **刷新 TLB**：寫入 `satp` 後自動刷新 TLB
5. **跳轉核心**：使用核心堆疊，進入 C/Rust 層級的陷阱處理器

回歸使用者模式時反向操作：儲存核心暫存器、恢復使用者頁表、`sret` 回到使用者程式碼。

## Trampoline 頁面的安全屬性

trampoline 頁面在使用者空間被映射為 **執行但不可寫入**（R-X），防止使用者程式碼修改切換程式。核心視角中則映射為 **讀寫執行**（RWX），讓核心可以在需要時更新 trampoline 程式碼。

## 相關文件

- [kernelvec.md](./kernelvec.md) — 核心陷阱向量
- [trap.md](./trap.md) — 陷阱處理框架
- [vm.md](./vm.md) — 虛擬記憶體管理
- [riscv.md](./riscv.md) — RISC-V 分頁機制
