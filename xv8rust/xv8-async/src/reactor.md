# Reactor — 事件驅動核心

`reactor.rs` 實作 xv8 async runtime 的 reactor 元件，負責監控 I/O 事件並喚醒等待的 Future。

## Reactor 模式

Reactor 是事件驅動架構的核心組件：

```
Future poll(I/O 未就緒)
  → 向 reactor 註冊興趣事件 (fd, 事件類型)
  → reactor 加入 epoll 監控列表
  → yield (return Poll::Pending)

epoll_wait 返回 (事件到達)
  → reactor 查找對應的 Waker
  → 呼叫 waker.wake() 重新排程 Future
  → Future 再次 poll (I/O 已就緒)
  → return Poll::Ready(data)
```

## Epoll 基礎

xv8 的 reactor 基於 epoll 系統呼叫。與傳統 `poll`/`select` 不同，epoll 在加入/移除監控 fd 時維持 O(1) 複雜度，並使用事件驅動而非輪詢。

## 相關文件

- [lib.md](./lib.md) — 非同步執行期總覽
- [io_async.md](./io_async.md) — 非同步 I/O 抽象
- [poll.md](../../kernel/src/poll.md) — 核心 poll/epoll 實作
