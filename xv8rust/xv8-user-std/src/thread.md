# Thread — 執行緒管理

`thread.rs` 實作 `std::thread` 模組，提供執行緒建立、睡眠、Join 等操作。

## 執行緒模型

xv8 執行緒模型遵循 1:1 模式——每個使用者執行緒對應一個核心排程單位：

- **thread::spawn**: 透過 `clone` 系統呼叫建立新執行緒，標誌 `CLONE_VM | CLONE_THREAD` 讓新行程共用位址空間與行程群組
- **thread::sleep**: 使用 `nanosleep` 系統呼叫讓當前執行緒進入休眠
- **JoinHandle**: 封裝 `waitpid` 或 `futex`，等待執行緒終止

## 執行緒區域儲存

TLS（Thread Local Storage）透過核心的執行緒指標暫存器（tp）支援。每個執行緒擁有獨立的 TLS 區塊，儲存 `thread_local!` 靜態變數。

## 相關文件

- [sync.md](./sync.md) — 同步原語
- [proc.md](../../kernel/src/proc.md) — 行程管理
- [thread.md](../../user/testbin/thread.md) — 執行緒測試
