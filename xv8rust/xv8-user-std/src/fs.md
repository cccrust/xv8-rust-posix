# Fs — 檔案系統操作

`fs.rs` 實作 `std::fs` 模組，提供檔案與目錄的操作抽象：`File::open`、`File::create`、`read_to_string`、`write`、`metadata`、`remove_file`、`create_dir`、`read_dir` 等。

## 系統呼叫映射

xv8 的 fs 模組將標準檔案操作委派給 xv8-libc 的系統呼叫：

| std::fs 函式 | 系統呼叫 |
|-------------|---------|
| File::open | open(path, flags) |
| File::read | read(fd, buf, count) |
| File::write | write(fd, buf, count) |
| metadata | stat(path, stat_buf) |
| remove_file | unlink(path) |
| create_dir | mkdir(path, mode) |
| read_dir | open + readdir |

## xv8 的適應

xv8 的檔案系統是 xv6 風格的 log-structured FS，支援 inode、block 與目錄架構。xv8-user-std 的 fs 模組可能不支援所有進階功能（如 symlink、檔案鎖定）。

## 相關文件

- [fs.md](../../kernel/src/fs.md) — 核心檔案系統
- [io.md](./io.md) — I/O 抽象
- [path.md](./path.md) — 路徑處理
- [xv8-user-std.md](../xv8-user-std.md) — 總覽
