# Kalloc — 實體頁面分配器

## 概述

`kalloc` 管理 xv8 核心的實體記憶體頁面（page），負責分配與回收 4 KiB 頁框（page frame）。這是虛擬記憶體系統的底層基礎——所有頁表建立、使用者記憶體分配、核心堆疊分配都依賴此模組。

## Buddy 分配演算法

xv8 使用經典的 buddy allocator（夥伴系統）管理實體記憶體：

```
Order 0: [ 4K ][ 4K ][ 4K ][ 4K ] ...
Order 1: [      8K      ][      8K      ] ...
Order 2: [             16K              ] ...
```

- 頁面以 **order**（2^order 個連續頁面）為單位管理
- 空閒頁面依 order 存放在鍊錶陣列中
- 分配請求會向上取整到最小的可滿足 order
- 回收時檢查 buddy（位址相鄰的同 order 頁塊）是否空閒，若是則合併為更高 order

### Buddy 配對規則

兩個 buddy 頁塊的實體位址相差 `2^order * PAGE_SIZE`，且由同一個 parent 頁塊分割而來。合併條件是：兩個頁塊皆空閒且互為 buddy。這種遞迴合併確保了大塊連續記憶體的存在。

## kalloc / kfree

- `kalloc()`: 從 physical page freelist 取出一個頁面，回傳實體位址。若無可用頁面則 panic（核心無法處理記憶體不足）。
- `kfree(pa)`: 將頁面歸還給 buddy allocator，觸發可能的合併。

## 初始記憶體偵測

核心在啟動時從 QEMU 的裝置樹（device tree）或 `virtio` 配置讀取實體記憶體大小，隨後將所有可用頁面初始化並加入 buddy 系統。核心自身使用的記憶體透過 `end` 標記排除。

## 相關文件

- [vm.md](./vm.md) — 虛擬記憶體管理
- [memlayout.md](./memlayout.md) — 核心記憶體佈局
