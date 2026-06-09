# Printf — 核心格式化輸出

## 概述

`printf` 是 xv8 核心的最小化格式化輸出函式，用於除錯與狀態輸出。由於核心無法依賴標準函式庫，`printf` 直接操作 UART（序列埠）將字元輸出到 QEMU 的除錯終端機。

## 格式化實作

`printf` 實作 Rust 的 `write!` 巨集介面，支援以下格式符：

- `%d` / `%i` — 十進位整數
- `%x` / `%p` — 十六進位格式
- `%s` — 字串
- `%c` — 單一字元
- `%%` — 跳脫百分比符號

格式化引擎逐字元解析格式字串，遇到 `%` 時解析型別標記，將對應參數轉為字串後透過 `console_putchar` 輸出。

## 與 UART 的互動

每次字元輸出最終呼叫 UART 驅動的 `putchar`，該函式等待傳送緩衝區空閒（TX FIFO 非滿狀態），將字元寫入 UART 暫存器。QEMU 將 UART 輸出轉向到 host 的終端機或 `-serial` 指定的檔案。

## 鎖定機制

核心可能存在多個 CPU 同時呼叫 `printf`，因此 `printf` 內部使用 spinlock 保護序列埠，確保輸出不會交錯混亂。

## 相關文件

- [uart.md](./uart.md) — UART 驅動詳細說明
- [console.md](./console.md) — 控制臺輸入輸出
