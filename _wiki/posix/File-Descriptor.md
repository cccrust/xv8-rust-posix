# File Descriptor — 檔案描述符

檔案描述符是 POSIX 用於代表開啟檔案、管道、裝置的小型非負整數。

## 概述

```
程式
  │
  ▼
open("/dev/null")  // 返回 3
  │                  │
  │                  ▼
  │               核心中的 struct file* 指向 inode
  │                  │
  ▼                  ▼
read(fd=3, ...)    讀取 /dev/null
```

## 標準描述符

| FD | 符號常數 | 用途 |
|----|----------|------|
| 0 | STDIN_FILENO | 標準輸入 |
| 1 | STDOUT_FILENO | 標準輸出 |
| 2 | STDERR_FILENO | 標準錯誤 |

## xv8 的 fd 結構

```rust
pub struct Process {
    pub fd_table: [Option<FileDesc>; 16],  // 每程序最多 16 個 fd
}

pub struct FileDesc {
    pub file: File,      // 指向開啟的檔案
    pub flags: u32,      // O_CLOEXEC, O_NONBLOCK 等
    pub mode: u32,       // f_mode
}
```

## open 返回值

```rust
pub fn open(path: *const u8, mode: i32) -> i32 {
    // 成功：返回新的 fd（最小可用）
    // 失敗：返回 -1
}
```

## 檔案描述符特性

### 繼承

fork 後，子程序繼承父程序的 fd：

```c
int fd = open("file.txt", O_RDONLY);
if (fork() == 0) {
    // 子程序也有 fd 指向同一個檔案
    read(fd, ...);  // 讀取相同檔案
}
```

### close-on-exec

設定後，exec 會關閉 fd：

```rust
// Rust (fcntl)
fcntl(fd, F_SETFD, FD_CLOEXEC);
```

## dup — 複製描述符

```c
int newfd = dup(oldfd);  // 複製到最小可用 fd
int newfd = dup2(oldfd, targetfd);  // 複製到指定 fd
```

用途：
- 重定向標準輸入/輸出
- 關閉特定 fd

```c
// 重定向 stdout 到 file
close(STDOUT_FILENO);
dup(fd);  // fd 成為新的 stdout
```

## 與 xv6 的差異

| 特性 | xv6 | xv8 |
|------|-----|-----|
| fd 表大小 | 16 | 16 |
| O_CLOEXEC | 無 | 有 |
| 檔案描述符 flags | 無 | 有 |
| fd 複製 | dup | dup/dup2 |

## read/write

```c
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);
```

### 返回值

| 返回值 | 意義 |
|--------|------|
| > 0 | 成功讀取/寫入的位元組數 |
| 0 | EOF（讀取）或無阻塞寫入 |
| -1 | 錯誤 |

## lseek

```c
off_t lseek(int fd, off_t offset, int whence);
// whence: SEEK_SET, SEEK_CUR, SEEK_END
```

## pipe

建立匿名管道：

```c
int p[2];
pipe(p);
// p[0] = 讀取端
// p[1] = 寫入端
```

## socket

建立網路通訊端：

```c
int sock = socket(AF_INET, SOCK_STREAM, 0);
// 返回 socket fd（與檔案 fd 同一機制）
```

## select/poll/epoll

多路復用 I/O：

```c
fd_set readfds;
FD_ZERO(&readfds);
FD_SET(fd, &readfds);
select(fd+1, &readfds, NULL, NULL, NULL);
```

## 底層機制

```
使用者程式
    │
    ├── read(fd, buf, n)
    │       │
    │       ▼
    └─── 核心：從 fd_table[fd] 取得 file*
            │
            ▼
        file->inode->read()
            │
            ▼
        硬體讀取
```

## 常見錯誤

| 錯誤 | 原因 |
|------|------|
| EBADF | fd 未開啟或已關閉 |
| EINTR | 系統呼叫被訊號中斷 |
| EIO | 實體 I/O 錯誤 |

## 關閉

```c
close(fd);
```

關閉後：
- 釋放 fd 給未來使用
- 檔案 table 參數計數減一
- 最後關閉時釋放 inode

## 相關主題

- [[Process]]：fd 在程序建立時的處理
- [[File-System]]：fd 如何指向 inode
- [[Pipe]]：管道的檔案描述符機制