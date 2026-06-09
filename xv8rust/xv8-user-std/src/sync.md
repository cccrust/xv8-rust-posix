# Sync — 同步原語

`sync.rs` 實作 `std::sync` 模組的同步原語：`Mutex`、`Condvar`（條件變數）、`Once`（一次性初始化）、`Barrier`（屏障）。

## 底層機制

xv8 的同步原語基於核心提供的 `futex`（Fast Userspace Mutex）系統呼叫。Futex 是一種高效的使用者空間鎖定機制：

1. 無競爭時：完全在使用者空間完成（原子操作）
2. 有競爭時：陷入核心，等待隊列

## 實作的類型

- **Mutex**: 互斥鎖，保護共享資源的獨佔訪問
- **Condvar**: 條件變數，允許執行緒等待特定條件成立
- **Once**: 一次性初始化，用於全域常數的懶初始化
- **Barrier**: 屏障，等待多個執行緒到達同步點

## xv8 的適應

在單 CPU 環境中，Mutex 的行為退化為純粹的排程標記（無真正競爭），但仍保持正確的排除語義。

## 相關文件

- [spinlock.md](../../kernel/src/spinlock.md) — 核心自旋鎖
- [sleeplock.md](../../kernel/src/sleeplock.md) — 核心睡眠鎖
- [thread.md](./thread.md) — 執行緒管理
