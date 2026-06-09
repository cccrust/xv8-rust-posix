# Io Async — 非同步 I/O 抽象

`io_async.rs` 提供非同步版本的 I/O 抽象，基於 Rust 的 `async`/`.await` 語法與 `Future` trait。

## 非同步 I/O 模型

與同步 I/O 不同，非同步 I/O 不會阻塞當前執行緒。xv8 的 async I/O 透過 reactor 模式實作：

1. 發起 I/O 操作（如 `read`）回傳 `Future<Output = io::Result<usize>>`
2. Future 被 poll 時，若操作未就緒則註冊到 reactor 的興趣列表
3. reactor 透過 epoll 監控 fd 事件，事件到來時喚醒對應的 Future
4. Future 再次 poll 完成 I/O 操作

## 實作的抽象

- **AsyncRead**: 非同步讀取 trait
- **AsyncWrite**: 非同步寫入 trait
- **AsyncReadExt/AsyncWriteExt**: extension trait 提供的便利方法

## 相關文件

- [lib.md](./lib.md) — 非同步執行期總覽
- [reactor.md](./reactor.md) — Reactor 事件驅動
- [io.md](../../xv8-user-std/src/io.md) — 同步 I/O 抽象
