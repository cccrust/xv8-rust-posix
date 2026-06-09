# Kernelvec — 核心陷阱向量

## 概述

`kernelvec.rs` 包含核心模式的陷阱向量（trap vector）處理程式，以組合語言撰寫（透過 Rust 的 `global_asm!` 嵌入）。陷阱向量是 CPU 在發生例外或中斷時跳轉的入口點，`stvec` CSR 指向此處。

## 三種陷阱向量

xv8 定義了三種不同的陷阱處理路徑：

### kernelvec
當 CPU 在 **核心模式** 下發生中斷或例外時觸發。由於核心已使用核心頁表與核心堆疊，處理器直接跳轉到 `kernelvec`，儲存暫存器到核心陷阱幀（trapframe），然後呼叫 `trap_from_kernel()`。典型情況包括：時鐘中斷、磁碟中斷、核心除錯例外。

### uservec
當 CPU 在 **使用者模式** 下觸發陷阱時使用（通常透過 `ecall` 指令）。`stvec` 在核心執行程式碼時會暫時切換。使用者陷阱需要切換頁表（從使用者頁表到核心頁表）並切換堆疊。

### timervec
專用於 **時鐘中斷** 的快速路徑。RISC-V 的 `stimecmp` CSR 在計數器超過比較值時觸發中斷。`timervec` 僅更新排程時間片並觸發排程器，不做完整暫存器儲存還原以減少延遲。

## Trapframe 佈局

trapframe 是儲存 CPU 暫存器狀態的記憶體區域，讓陷阱處理器可以暫停當前執行緒，處理事件後精確恢復。`kernelvec` 依 RISC-V ABI 儲存所有 32 個通用暫存器（x0–x31），其中 x0（zero）為硬體連線的零值，不需實際儲存。

## 相關文件

- [trap.md](./trap.md) — 陷阱處理框架
- [trampoline.md](./trampoline.md) — 使用者/核心模式切換
- [start.md](./start.md) — 初始陷阱設定
- [riscv.md](./riscv.md) — RISC-V 特權架構
